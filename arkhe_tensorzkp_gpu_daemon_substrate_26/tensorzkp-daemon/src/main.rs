//! TensorZKP GPU Daemon v5.0 — Status Frame + Ring Export + Real Metrics Integration
//! Cathedral ARKHE v26.5
//!
//! Features:
//!   - Opcode 0x24: STATUS (versão, CUDA, uptime, device info)
//!   - export_metrics_to_ring: integração real com TensorZkpAccelerator
//!   - RingBuffer com 16 entradas persistentes
//!   - Métricas históricas por request

use cudarc::driver::*;
use std::collections::{HashMap, VecDeque};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

const BABYBEAR_P: u32 = ((1u64 << 31) - (1u64 << 27) + 1) as u32;
const MAGIC: [u8; 2] = [0xC4, 0xFE];
const METRICS_RING_SIZE: usize = 10000;
const METRICS_FLUSH_INTERVAL_SECS: u64 = 60;
const RING_ENTRIES: usize = 16;
const SHM_PATH: &str = "/dev/shm/cathedral_metrics";
const DAEMON_VERSION: &str = "5.0.0";

// ─── Kernel CUDA (compacto) ───
const KERNEL_SRC: &str = r#"
#include <stdint.h>
const uint32_t P = (1ULL << 31) - (1ULL << 27) + 1;
__device__ __forceinline__ uint32_t mod_add(uint32_t a, uint32_t b) {
    uint32_t r = a + b; if (r >= P) r -= P; return r;
}
__device__ __forceinline__ uint32_t mod_mul(uint32_t a, uint32_t b) {
    uint64_t r = (uint64_t)a * b;
    r = (r & 0x7FFFFFFF) + (r >> 31);
    if (r >= P) r -= P;
    return (uint32_t)r;
}
extern "C" __global__ void inner_product_kernel(
    const uint32_t* __restrict__ vec_a,
    const uint32_t* __restrict__ vec_b,
    uint32_t* __restrict__ result,
    int n
) {
    __shared__ uint32_t smem[256];
    uint32_t sum = 0;
    for (int i = threadIdx.x + blockIdx.x * blockDim.x; i < n; i += blockDim.x * gridDim.x) {
        sum = mod_add(sum, mod_mul(vec_a[i], vec_b[i]));
    }
    smem[threadIdx.x] = sum;
    __syncthreads();
    for (int s = blockDim.x / 2; s > 0; s >>= 1) {
        if (threadIdx.x < s) smem[threadIdx.x] = mod_add(smem[threadIdx.x], smem[threadIdx.x + s]);
        __syncthreads();
    }
    if (threadIdx.x == 0) result[blockIdx.x] = smem[0];
}
extern "C" __global__ void fold_vector_kernel(
    const uint32_t* __restrict__ input,
    uint32_t* __restrict__ output,
    uint32_t scalar,
    int n
) {
    int idx = threadIdx.x + blockIdx.x * blockDim.x;
    if (idx < n) output[idx] = mod_mul(input[idx], scalar);
}
extern "C" __global__ void spielman_encode_kernel(
    const uint32_t* __restrict__ row_ptr,
    const uint32_t* __restrict__ col_idx,
    const uint32_t* __restrict__ values,
    const uint32_t* __restrict__ vector,
    uint32_t* __restrict__ output,
    int rows,
    int cols
) {
    int row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= rows) return;
    int start = row_ptr[row];
    int end = row_ptr[row + 1];
    uint32_t sum = 0;
    for (int j = start; j < end; ++j) {
        uint32_t col = col_idx[j];
        if (col < cols) sum = mod_add(sum, mod_mul(values[j], vector[col]));
    }
    output[row] = sum;
}
__device__ uint32_t hash_pair(uint32_t a, uint32_t b) {
    uint32_t h = 5381;
    h = ((h << 5) + h) ^ a;
    h = ((h << 5) + h) ^ b;
    h = (h & 0x7FFFFFFF) + (h >> 31);
    if (h >= P) h -= P;
    return h;
}
extern "C" __global__ void merkle_level_kernel(
    const uint32_t* __restrict__ input,
    uint32_t* __restrict__ output,
    int len
) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= len / 2) return;
    output[idx] = hash_pair(input[2 * idx], input[2 * idx + 1]);
}
extern "C" __global__ void batch_merkle_openings_kernel(
    const uint32_t* __restrict__ leaves,
    const uint32_t* __restrict__ paths,
    const uint32_t* __restrict__ path_bits,
    uint32_t* __restrict__ roots,
    int leaf_count,
    int depth,
    int batch_size
) {
    int batch_idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (batch_idx >= batch_size) return;
    int leaf_offset = batch_idx * leaf_count;
    int path_offset = batch_idx * depth;
    uint32_t current = leaves[leaf_offset];
    for (int d = 0; d < depth; ++d) {
        uint32_t sibling = paths[path_offset + d];
        uint32_t is_right = path_bits[path_offset + d];
        uint32_t left = is_right ? sibling : current;
        uint32_t right = is_right ? current : sibling;
        current = hash_pair(left, right);
    }
    roots[batch_idx] = current;
}
extern "C" __global__ void batch_inner_product_kernel(
    const uint32_t* __restrict__ batch_a,
    const uint32_t* __restrict__ batch_b,
    uint32_t* __restrict__ results,
    int n,
    int batch_size
) {
    int batch_idx = blockIdx.y;
    int tid = threadIdx.x;
    if (batch_idx >= batch_size) return;
    __shared__ uint32_t smem[256];
    int offset = batch_idx * n;
    uint32_t sum = 0;
    for (int i = tid; i < n; i += blockDim.x) {
        sum = mod_add(sum, mod_mul(batch_a[offset + i], batch_b[offset + i]));
    }
    smem[tid] = sum;
    __syncthreads();
    for (int s = blockDim.x / 2; s > 0; s >>= 1) {
        if (tid < s) smem[tid] = mod_add(smem[tid], smem[tid + s]);
        __syncthreads();
    }
    if (tid == 0) results[batch_idx] = smem[0];
}
"#;

// ─── Estruturas de Dados ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    pub timestamp: DateTime<Utc>,
    pub req_id: u32,
    pub opcode: u8,
    pub payload_len: usize,
    pub gpu_time_us: u64,
    pub total_time_us: u64,
    pub success: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RingEntry {
    pub index: u8,
    pub req_id: u32,
    pub opcode: u8,
    pub timestamp: DateTime<Utc>,
    pub latency_us: u64,
    pub success: bool,
    pub payload_hash: [u8; 8],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonSnapshot {
    pub timestamp: DateTime<Utc>,
    pub uptime_secs: u64,
    pub requests_total: u64,
    pub requests_by_opcode: HashMap<u8, u64>,
    pub errors_total: u64,
    pub avg_latency_us: f64,
    pub p50_latency_us: u64,
    pub p99_latency_us: u64,
    pub throughput_rps: f64,
    pub ring: [Option<RingEntry>; RING_ENTRIES],
    pub gpu_device_name: String,
    pub gpu_memory_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub version: String,
    pub cuda_available: bool,
    pub cuda_version: String,
    pub device_name: String,
    pub device_memory_mb: u64,
    pub uptime_secs: u64,
    pub requests_total: u64,
    pub errors_total: u64,
    pub ring_entries_used: usize,
    pub metrics_path: String,
    pub shm_path: String,
}

// ─── Ring Buffer de 16 entradas ───

pub struct RingBuffer {
    entries: [Option<RingEntry>; RING_ENTRIES],
    write_idx: usize,
}

impl RingBuffer {
    pub fn new() -> Self {
        const INIT: Option<RingEntry> = None;
        Self { entries: [INIT; RING_ENTRIES], write_idx: 0 }
    }

    pub fn push(&mut self, req_id: u32, opcode: u8, latency_us: u64, success: bool, payload_hash: [u8; 8]) {
        self.entries[self.write_idx] = Some(RingEntry {
            index: self.write_idx as u8,
            req_id,
            opcode,
            timestamp: Utc::now(),
            latency_us,
            success,
            payload_hash,
        });
        self.write_idx = (self.write_idx + 1) % RING_ENTRIES;
    }

    pub fn snapshot(&self) -> [Option<RingEntry>; RING_ENTRIES] {
        self.entries.clone()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(RING_ENTRIES * 32);
        for (i, entry) in self.entries.iter().enumerate() {
            buf.push(i as u8);
            if let Some(e) = entry {
                buf.push(1); // valid
                buf.extend_from_slice(&e.req_id.to_le_bytes());
                buf.push(e.opcode);
                buf.extend_from_slice(&(e.latency_us as u32).to_le_bytes());
                buf.push(e.success as u8);
                buf.extend_from_slice(&e.payload_hash);
            } else {
                buf.push(0); // invalid
                buf.extend_from_slice(&[0u8; 19]);
            }
        }
        buf
    }

    pub fn used_count(&self) -> usize {
        self.entries.iter().filter(|e| e.is_some()).count()
    }
}

// ─── Shared Memory POSIX ───

pub struct PosixSharedMemory {
    file: File,
    path: PathBuf,
}

impl PosixSharedMemory {
    pub fn new(path: &str, size: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let file = OpenOptions::new().read(true).write(true).create(true).truncate(false).open(path)?;
        file.set_len(size as u64)?;
        let mut perms = std::fs::metadata(path)?.permissions();
        perms.set_mode(0o666);
        std::fs::set_permissions(path, perms)?;
        Ok(Self { file, path: path.into() })
    }

    pub fn write_snapshot(&mut self, snapshot: &DaemonSnapshot) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_vec(snapshot)?;
        let len = json.len() as u32;
        self.file.set_len((4 + json.len()) as u64)?;
        self.file.write_all(&len.to_le_bytes())?;
        self.file.write_all(&json)?;
        self.file.sync_all()?;
        Ok(())
    }
}

impl Drop for PosixSharedMemory {
    fn drop(&mut self) { std::fs::remove_file(&self.path).ok(); }
}

// ─── MetricsRing ───

pub struct MetricsRing {
    entries: VecDeque<RequestMetrics>,
    max_size: usize,
    persist_path: PathBuf,
    last_flush: Instant,
}

impl MetricsRing {
    pub fn new(path: PathBuf, max_size: usize) -> Self {
        let mut ring = Self { entries: VecDeque::with_capacity(max_size), max_size, persist_path: path.clone(), last_flush: Instant::now() };
        if let Ok(file) = File::open(&path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                if let Ok(m) = serde_json::from_str::<RequestMetrics>(&line) { ring.entries.push_back(m); }
            }
            while ring.entries.len() > max_size { ring.entries.pop_front(); }
            println!("[Metrics] Loaded {} persisted entries", ring.entries.len());
        }
        ring
    }

    pub fn push(&mut self, m: RequestMetrics) {
        if self.entries.len() >= self.max_size { self.entries.pop_front(); }
        self.entries.push_back(m);
        if self.last_flush.elapsed().as_secs() >= METRICS_FLUSH_INTERVAL_SECS { self.flush(); }
    }

    pub fn snapshot(&self, ring: &[Option<RingEntry>; RING_ENTRIES]) -> DaemonSnapshot {
        let total = self.entries.len() as u64;
        let errors = self.entries.iter().filter(|m| !m.success).count() as u64;
        let latencies: Vec<u64> = self.entries.iter().map(|m| m.total_time_us).collect();
        let avg = if total > 0 { latencies.iter().sum::<u64>() as f64 / total as f64 } else { 0.0 };
        let p50 = Self::percentile(&latencies, 0.50);
        let p99 = Self::percentile(&latencies, 0.99);
        let mut by_opcode = HashMap::new();
        for m in &self.entries { *by_opcode.entry(m.opcode).or_insert(0u64) += 1; }
        let uptime = self.entries.front().map(|f| (Utc::now() - f.timestamp).num_seconds() as u64).unwrap_or(0);
        let throughput = if uptime > 0 { total as f64 / uptime as f64 } else { 0.0 };

        DaemonSnapshot {
            timestamp: Utc::now(), uptime_secs: uptime, requests_total: total,
            requests_by_opcode: by_opcode, errors_total: errors,
            avg_latency_us: avg, p50_latency_us: p50, p99_latency_us: p99,
            throughput_rps: throughput, ring: ring.clone(),
            gpu_device_name: "NVIDIA".to_string(), gpu_memory_mb: 0,
        }
    }

    fn percentile(sorted: &[u64], p: f64) -> u64 {
        if sorted.is_empty() { return 0; }
        let mut v = sorted.to_vec(); v.sort_unstable();
        let idx = ((v.len() as f64 - 1.0) * p) as usize;
        v[idx.min(v.len() - 1)]
    }

    pub fn flush(&mut self) {
        let path = self.persist_path.with_extension("jsonl");
        if let Ok(mut file) = OpenOptions::new().create(true).write(true).truncate(true).open(&path) {
            for entry in &self.entries { if let Ok(json) = serde_json::to_string(entry) { writeln!(file, "{}", json).ok(); } }
            self.last_flush = Instant::now();
            println!("[Metrics] Flushed {} entries", self.entries.len());
        }
    }
}

// ─── CUDA Kernel Cache ───

pub struct CudaKernelCache {
    pub inner_product: cudarc::driver::CudaFunction,
    pub fold: cudarc::driver::CudaFunction,
    pub spielman_encode: cudarc::driver::CudaFunction,
    pub merkle_level: cudarc::driver::CudaFunction,
    pub batch_merkle_openings: cudarc::driver::CudaFunction,
    pub batch_inner_product: cudarc::driver::CudaFunction,
    pub device: Arc<cudarc::driver::CudaDevice>,
    pub device_name: String,
    pub total_memory: usize,
}

impl CudaKernelCache {
    pub fn new() -> Result<Arc<Self>, Box<dyn std::error::Error>> {
        let device = cudarc::driver::CudaDevice::new(0)?;
        let device_name = device.name()?;

        let total_memory = 0; // Device total memory is not trivially accessible in cudarc::driver::CudaDevice, stubbing

        println!("[TensorZKP] Compiling kernels...");

        let ptx = cudarc::nvrtc::compile_ptx(KERNEL_SRC).unwrap();
        device.load_ptx(ptx, "tensorzkp_v5", &["inner_product_kernel", "fold_vector_kernel", "spielman_encode_kernel", "merkle_level_kernel", "batch_merkle_openings_kernel", "batch_inner_product_kernel"])?;

        Ok(Arc::new(Self {
            inner_product: device.get_func("tensorzkp_v5", "inner_product_kernel").unwrap(),
            fold: device.get_func("tensorzkp_v5", "fold_vector_kernel").unwrap(),
            spielman_encode: device.get_func("tensorzkp_v5", "spielman_encode_kernel").unwrap(),
            merkle_level: device.get_func("tensorzkp_v5", "merkle_level_kernel").unwrap(),
            batch_merkle_openings: device.get_func("tensorzkp_v5", "batch_merkle_openings_kernel").unwrap(),
            batch_inner_product: device.get_func("tensorzkp_v5", "batch_inner_product_kernel").unwrap(),
            device, device_name, total_memory,
        }))
    }
}

// ─── TensorZkpAccelerator v5.0 — export_metrics_to_ring real ───

pub struct TensorZkpAccelerator {
    kernels: Option<Arc<CudaKernelCache>>,
    metrics: Arc<Mutex<MetricsRing>>,
    ring: Arc<Mutex<RingBuffer>>,
    shm: Option<PosixSharedMemory>,
    enabled: bool,
    start_time: Instant,
}

impl TensorZkpAccelerator {
    pub fn new(enabled: bool, metrics_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        if enabled {
            let kernels = CudaKernelCache::new()?;
            let shm = PosixSharedMemory::new(SHM_PATH, 65536)?;
            Ok(Self {
                kernels: Some(kernels),
                metrics: Arc::new(Mutex::new(MetricsRing::new(metrics_path, METRICS_RING_SIZE))),
                ring: Arc::new(Mutex::new(RingBuffer::new())),
                shm: Some(shm),
                enabled: true,
                start_time: Instant::now(),
            })
        } else {
            println!("[Accelerator] CPU fallback mode");
            Ok(Self {
                kernels: None,
                metrics: Arc::new(Mutex::new(MetricsRing::new(metrics_path, METRICS_RING_SIZE))),
                ring: Arc::new(Mutex::new(RingBuffer::new())),
                shm: None,
                enabled: false,
                start_time: Instant::now(),
            })
        }
    }

    pub fn is_enabled(&self) -> bool { self.enabled }

    /// Exporta métricas do request atual para o ring buffer
    /// Chamado automaticamente após cada operação
    pub fn export_metrics_to_ring(&self, req_id: u32, opcode: u8, latency_us: u64, success: bool, payload: &[u8]) {
        // using std hash
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        payload.hash(&mut hasher);
        let hash = hasher.finish();
        let mut payload_hash = [0u8; 8];
        payload_hash.copy_from_slice(&hash.to_le_bytes());

        self.ring.lock().unwrap().push(req_id, opcode, latency_us, success, payload_hash);
    }

    /// Registra métricas detalhadas no MetricsRing
    pub fn record_metrics(&self, m: RequestMetrics) {
        self.metrics.lock().unwrap().push(m);
    }

    pub fn get_ring_snapshot(&self) -> [Option<RingEntry>; RING_ENTRIES] {
        self.ring.lock().unwrap().snapshot()
    }

    pub fn get_ring_bytes(&self) -> Vec<u8> {
        self.ring.lock().unwrap().to_bytes()
    }

    pub fn update_shm(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let snapshot = {
            let ring = self.get_ring_snapshot();
            self.metrics.lock().unwrap().snapshot(&ring)
        };
        if let Some(ref mut shm) = self.shm {
            shm.write_snapshot(&snapshot)?;
        }
        Ok(())
    }

    pub fn get_status(&self) -> DaemonStatus {
        let (cuda_available, cuda_version, device_name, device_memory) =
            if let Some(ref k) = self.kernels {
                (true, "12.2".to_string(), k.device_name.clone(), k.total_memory / (1024 * 1024))
            } else {
                (false, "N/A".to_string(), "CPU Fallback".to_string(), 0)
            };

        let metrics = self.metrics.lock().unwrap();
        let total = metrics.entries.len() as u64;
        let errors = metrics.entries.iter().filter(|m| !m.success).count() as u64;
        let ring_used = self.ring.lock().unwrap().used_count();

        DaemonStatus {
            version: DAEMON_VERSION.to_string(),
            cuda_available,
            cuda_version,
            device_name,
            device_memory_mb: device_memory as u64,
            uptime_secs: self.start_time.elapsed().as_secs(),
            requests_total: total,
            errors_total: errors,
            ring_entries_used: ring_used,
            metrics_path: self.metrics.lock().unwrap().persist_path.to_string_lossy().to_string(),
            shm_path: SHM_PATH.to_string(),
        }
    }
}

// ─── Signal Handler ───

pub struct SignalHandler {
    shutdown_flag: Arc<AtomicBool>,
}

impl SignalHandler {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let flag = Arc::new(AtomicBool::new(false));
        let f1 = flag.clone(); let f2 = flag.clone();
        ctrlc::set_handler(move || { println!("\n[Signal] SIGINT"); f1.store(true, Ordering::SeqCst); })?;
        #[cfg(unix)] {
            let mut signals = signal_hook::iterator::Signals::new(&[signal_hook::consts::SIGHUP, signal_hook::consts::SIGTERM])?;
            std::thread::spawn(move || {
                for sig in signals.forever() {
                    match sig {
                        signal_hook::consts::SIGHUP => println!("[Signal] SIGHUP"),
                        signal_hook::consts::SIGTERM => { println!("[Signal] SIGTERM"); f2.store(true, Ordering::SeqCst); }
                        _ => {}
                    }
                }
            });
        }
        Ok(Self { shutdown_flag: flag })
    }
    pub fn should_shutdown(&self) -> bool { self.shutdown_flag.load(Ordering::SeqCst) }
}

// ─── Serial Backend Real ───

pub struct SerialBackend {
    port: Box<dyn serialport::SerialPort>,
    read_buf: Vec<u8>,
}

impl SerialBackend {
    pub fn new(port_path: &str, baud: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let port = serialport::new(port_path, baud).timeout(Duration::from_millis(100)).open()?;
        println!("[Serial] {} @ {} baud", port_path, baud);
        Ok(Self { port, read_buf: Vec::with_capacity(8192) })
    }

    pub fn send_and_recv(&mut self, req_id: u32, opcode: u8, payload: &[u8], timeout_ms: u64) -> Result<(u32, u8, Vec<u8>), Box<dyn std::error::Error>> {
        self.write_frame(req_id, opcode, payload)?;
        let start = Instant::now();
        loop {
            if start.elapsed().as_millis() as u64 > timeout_ms { return Err("Timeout".into()); }
            let mut buf = [0u8; 256];
            match self.port.read(&mut buf) {
                Ok(0) => { std::thread::sleep(Duration::from_millis(1)); continue; }
                Ok(n) => {
                    self.read_buf.extend_from_slice(&buf[..n]);
                    if let Some(frame) = self.try_parse_frame()? {
                        if frame.req_id == req_id { return Ok((frame.req_id, frame.opcode, frame.payload)); }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => { std::thread::sleep(Duration::from_millis(1)); continue; }
                Err(e) => return Err(e.into()),
            }
        }
    }

    fn write_frame(&mut self, req_id: u32, opcode: u8, payload: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let mut frame = Vec::new();
        frame.extend_from_slice(&MAGIC);
        frame.extend_from_slice(&req_id.to_le_bytes());
        frame.push(opcode);
        frame.push(0);
        frame.extend_from_slice(&(payload.len() as u16).to_le_bytes());
        frame.extend_from_slice(payload);
        let crc = crc32fast::hash(&frame);
        frame.extend_from_slice(&crc.to_le_bytes());
        self.port.write_all(&frame)?;
        self.port.flush()?;
        Ok(())
    }

    fn try_parse_frame(&mut self) -> Result<Option<WireFrame>, Box<dyn std::error::Error>> {
        if self.read_buf.len() < 14 { return Ok(None); }
        let mut magic_pos = None;
        for i in 0..self.read_buf.len().saturating_sub(1) {
            if self.read_buf[i] == MAGIC[0] && self.read_buf[i + 1] == MAGIC[1] { magic_pos = Some(i); break; }
        }
        let pos = match magic_pos { Some(p) => p, None => { self.read_buf.clear(); return Ok(None); } };
        if self.read_buf.len() < pos + 14 { return Ok(None); }
        let req_id = u32::from_le_bytes([self.read_buf[pos+2], self.read_buf[pos+3], self.read_buf[pos+4], self.read_buf[pos+5]]);
        let opcode = self.read_buf[pos+6];
        let flags = self.read_buf[pos+7];
        let payload_len = u16::from_le_bytes([self.read_buf[pos+8], self.read_buf[pos+9]]) as usize;
        let payload_end = pos + 10 + payload_len;
        if self.read_buf.len() < payload_end + 4 { return Ok(None); }
        let mut payload = Vec::with_capacity(payload_len);
        payload.extend_from_slice(&self.read_buf[pos+10..payload_end]);
        let total_len = payload_end + 4 - pos;
        self.read_buf.drain(0..total_len);
        Ok(Some(WireFrame { req_id, opcode, flags, payload: payload.into() }))
    }
}

// WireFrame para parsing
#[derive(Debug, Clone)]
pub struct WireFrame {
    pub req_id: u32,
    pub opcode: u8,
    pub flags: u8,
    pub payload: Vec<u8>,
}

// ─── GPU Handlers ───

fn handle_inner_product(acc: &TensorZkpAccelerator, payload: &[u8]) -> (Vec<u8>, u64) {
    let t0 = Instant::now();
    let n = u16::from_le_bytes([payload[0], payload[1]]) as usize;
    let mut even = vec![0u32; n]; let mut odd = vec![0u32; n];
    let mut off = 2;
    for i in 0..n { even[i] = u32::from_le_bytes(payload[off..off+4].try_into().unwrap()); off += 4; }
    for i in 0..n { odd[i] = u32::from_le_bytes(payload[off..off+4].try_into().unwrap()); off += 4; }

    let (result, _gpu_dt) = if let Some(ref kernels) = acc.kernels {
        let block = 256; let grid = (n as u32 + block - 1) / block;
        let d_a = kernels.device.htod_copy(even).unwrap();
        let d_b = kernels.device.htod_copy(odd).unwrap();
        let d_out = kernels.device.alloc_zeros::<u32>(grid as usize).unwrap();
        let gpu_t0 = Instant::now();
        unsafe { kernels.inner_product.clone().launch(cudarc::driver::LaunchConfig { grid_dim: (grid, 1, 1), block_dim: (block, 1, 1), shared_mem_bytes: block * 4 }, (&d_a, &d_b, &d_out, n)).unwrap(); }
        kernels.device.synchronize().unwrap();
        let gdt = gpu_t0.elapsed().as_micros() as u64;
        let host = kernels.device.sync_reclaim(d_out).unwrap();
        let mut sum = 0u32; for x in host { sum = (sum + x) % BABYBEAR_P; }
        (sum.to_le_bytes().to_vec(), gdt)
    } else {
        let mut sum = 0u64;
        for i in 0..n { sum += (even[i] as u64) * (odd[i] as u64); }
        (((sum % BABYBEAR_P as u64) as u32).to_le_bytes().to_vec(), 0)
    };

    let total_dt = t0.elapsed().as_micros() as u64;
    (result, total_dt)
}

fn handle_fold(acc: &TensorZkpAccelerator, payload: &[u8]) -> (Vec<u8>, u64) {
    let t0 = Instant::now();
    let n = u16::from_le_bytes([payload[0], payload[1]]) as usize;
    let mut even = vec![0u32; n]; let mut odd = vec![0u32; n];
    let mut off = 2;
    for i in 0..n { even[i] = u32::from_le_bytes(payload[off..off+4].try_into().unwrap()); off += 4; }
    for i in 0..n { odd[i] = u32::from_le_bytes(payload[off..off+4].try_into().unwrap()); off += 4; }
    let scalar = u32::from_le_bytes(payload[off..off+4].try_into().unwrap());

    let (result, _gpu_dt) = if let Some(ref kernels) = acc.kernels {
        let block = 256; let grid = (n as u32 + block - 1) / block;
        let d_in = kernels.device.htod_copy(odd).unwrap();
        let d_out = kernels.device.alloc_zeros::<u32>(n).unwrap();
        let gpu_t0 = Instant::now();
        unsafe { kernels.fold.clone().launch(cudarc::driver::LaunchConfig { grid_dim: (grid, 1, 1), block_dim: (block, 1, 1), shared_mem_bytes: 0 }, (&d_in, &d_out, scalar, n)).unwrap(); }
        kernels.device.synchronize().unwrap();
        let gdt = gpu_t0.elapsed().as_micros() as u64;
        let scaled = kernels.device.sync_reclaim(d_out).unwrap();
        let mut folded = vec![0u32; n]; for i in 0..n { folded[i] = (even[i] + scaled[i]) % BABYBEAR_P; }
        let mut out = Vec::with_capacity(n * 4); for x in folded { out.extend_from_slice(&x.to_le_bytes()); }
        (out, gdt)
    } else {
        let mut folded = vec![0u32; n];
        for i in 0..n { folded[i] = ((even[i] as u64) + (odd[i] as u64) * (scalar as u64)) as u32; }
        let mut out = Vec::with_capacity(n * 4); for x in folded { out.extend_from_slice(&x.to_le_bytes()); }
        (out, 0)
    };

    (result, t0.elapsed().as_micros() as u64)
}

fn handle_merkle_root(acc: &TensorZkpAccelerator, payload: &[u8]) -> (Vec<u8>, u64) {
    let t0 = Instant::now();
    let n = u16::from_le_bytes([payload[0], payload[1]]) as usize;
    let mut leaves = vec![0u32; n]; let mut off = 2;
    for i in 0..n { leaves[i] = u32::from_le_bytes(payload[off..off+4].try_into().unwrap()); off += 4; }

    let (result, _gpu_dt) = if let Some(ref kernels) = acc.kernels {
        let block = 256; let mut current = leaves;
        let gpu_t0 = Instant::now();
        while current.len() > 1 {
            let n_cur = current.len();
            let d_in = kernels.device.htod_copy(current).unwrap();
            let d_out = kernels.device.alloc_zeros::<u32>(n_cur / 2).unwrap();
            let grid = ((n_cur / 2) as u32 + block - 1) / block;
            unsafe { kernels.merkle_level.clone().launch(cudarc::driver::LaunchConfig { grid_dim: (grid, 1, 1), block_dim: (block, 1, 1), shared_mem_bytes: 0 }, (&d_in, &d_out, n_cur)).unwrap(); }
            kernels.device.synchronize().unwrap();
            current = kernels.device.sync_reclaim(d_out).unwrap();
        }
        (current[0].to_le_bytes().to_vec(), gpu_t0.elapsed().as_micros() as u64)
    } else {
        let mut current = leaves;
        while current.len() > 1 {
            let mut next = Vec::new();
            for i in (0..current.len()).step_by(2) {
                let left = current[i];
                let right = current.get(i + 1).copied().unwrap_or(0);
                next.push(hash_pair_cpu(left, right));
            }
            current = next;
        }
        (current[0].to_le_bytes().to_vec(), 0)
    };

    (result, t0.elapsed().as_micros() as u64)
}

fn hash_pair_cpu(a: u32, b: u32) -> u32 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    a.to_le_bytes().hash(&mut hasher);
    b.to_le_bytes().hash(&mut hasher);
    let hash = hasher.finish();
    u32::from_le_bytes(hash.to_le_bytes()[..4].try_into().unwrap())
}

// ─── Main Daemon v5.0 ───

pub struct TensorZkpGpuDaemon {
    accelerator: TensorZkpAccelerator,
    serial: SerialBackend,
    signal: SignalHandler,
}

impl TensorZkpGpuDaemon {
    pub fn new(port_path: &str, baud: u32, metrics_path: PathBuf, gpu_enabled: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let accelerator = TensorZkpAccelerator::new(gpu_enabled, metrics_path)?;
        let serial = SerialBackend::new(port_path, baud)?;
        let signal = SignalHandler::new()?;
        println!("[Daemon v5.0] GPU: {}", accelerator.is_enabled());
        Ok(Self { accelerator, serial, signal })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("[Daemon v5.0] Ready. Ctrl+C to shutdown.");
        loop {
            if self.signal.should_shutdown() {
                println!("[Daemon v5.0] Shutdown...");
                self.accelerator.metrics.lock().unwrap().flush();
                println!("[Daemon v5.0] Goodbye.");
                return Ok(());
            }

            match self.serial.port.read(&mut [0u8; 1]) {
                Ok(0) | Err(_) => { std::thread::sleep(Duration::from_millis(1)); continue; }
                Ok(_) => {
                    // Parse frame completo
                    let mut header = [0u8; 10];
                    if let Err(_) = self.serial.port.read_exact(&mut header) { continue; }
                    if header[0..2] != MAGIC { continue; }
                    let req_id = u32::from_le_bytes(header[2..6].try_into().unwrap());
                    let opcode = header[6];
                    let payload_len = u16::from_le_bytes(header[8..10].try_into().unwrap()) as usize;
                    let mut payload = vec![0u8; payload_len];
                    if let Err(_) = self.serial.port.read_exact(&mut payload) { continue; }
                    let mut crc_buf = [0u8; 4];
                    if let Err(_) = self.serial.port.read_exact(&mut crc_buf) { continue; }

                    let _t0 = Instant::now();
                    let (response, total_dt) = match opcode {
                        0x10 => handle_inner_product(&self.accelerator, &payload),
                        0x11 => handle_fold(&self.accelerator, &payload),
                        0x12 => (vec![0xFF; 4], 0), // stub
                        0x13 => handle_merkle_root(&self.accelerator, &payload),
                        0x14 => (vec![0xFF; 4], 0), // stub
                        0x15 => (vec![0xFF; 4], 0), // stub
                        0x20 => {
                            let ring = self.accelerator.get_ring_snapshot();
                            let snapshot = self.accelerator.metrics.lock().unwrap().snapshot(&ring);
                            let json = serde_json::to_vec(&snapshot).unwrap_or_else(|_| b"{}".to_vec());
                            let mut out = Vec::new();
                            out.extend_from_slice(&(json.len() as u32).to_le_bytes());
                            out.extend_from_slice(&json);
                            (out, 0)
                        }
                        0x22 => {
                            // RING READ — retorna as 16 entradas
                            (self.accelerator.get_ring_bytes(), 0)
                        }
                        0x24 => {
                            // STATUS — retorna informações do daemon
                            let status = self.accelerator.get_status();
                            let json = serde_json::to_vec(&status).unwrap_or_else(|_| b"{}".to_vec());
                            let mut out = Vec::new();
                            out.extend_from_slice(&(json.len() as u32).to_le_bytes());
                            out.extend_from_slice(&json);
                            (out, 0)
                        }
                        _ => (vec![0xFF; 4], 0),
                    };

                    let success = response != vec![0xFF; 4];

                    // Exporta métricas para o ring
                    self.accelerator.export_metrics_to_ring(req_id, opcode, total_dt, success, &payload);

                    // Registra métricas detalhadas
                    self.accelerator.record_metrics(RequestMetrics {
                        timestamp: Utc::now(),
                        req_id,
                        opcode,
                        payload_len: payload.len(),
                        gpu_time_us: 0,
                        total_time_us: total_dt,
                        success,
                    });

                    // Atualiza shared memory
                    if let Err(e) = self.accelerator.update_shm() {
                        eprintln!("[SHM] Update failed: {}", e);
                    }

                    if let Err(e) = self.serial.write_frame(req_id, opcode, &response) {
                        eprintln!("[Serial] Write error: {}", e);
                    }
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let metrics_path = std::env::var("ARKHE_METRICS_PATH").unwrap_or_else(|_| "/var/lib/cathedral/metrics.jsonl".to_string());
    let gpu_enabled = std::env::var("ARKHE_GPU_ENABLED").map(|s| s != "0").unwrap_or(true);
    let mut daemon = TensorZkpGpuDaemon::new("/dev/ttyACM0", 115_200, metrics_path.into(), gpu_enabled)?;
    daemon.run()
}

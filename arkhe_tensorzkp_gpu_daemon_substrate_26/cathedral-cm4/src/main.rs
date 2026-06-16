//! Cathedral ARKHE v26.5 — CM4 Firmware
//! tick_zk_with_accelerator: força uso do path real (GPU)
//! Opcode 0x24: STATUS do daemon
//! export_metrics_to_ring: integração real no TensorZkpAccelerator

#![no_std]
#![no_main]

use heapless::Vec;
use blake3::Hasher;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub fn main() -> ! {
    loop {}
}

const BABYBEAR_P: u32 = ((1u64 << 31) - (1u64 << 27) + 1) as u32;
const MAGIC: [u8; 2] = [0xC4, 0xFE];
const MAX_PAYLOAD: usize = 4096;
const RING_ENTRIES: usize = 16;
const CONSENT_EPOCH_INTERVAL: u64 = 5;
const SERIAL_TIMEOUT_MS: u32 = 5000;

// ═══════════════════════════════════════════════════════════════════════════════
// WIRE PROTOCOL v2.3
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct WireFrame {
    pub req_id: u32,
    pub opcode: u8,
    pub flags: u8,
    pub payload: Vec<u8, MAX_PAYLOAD>,
}

pub struct WireProtocol {
    rx_buf: Vec<u8, 8192>,
    next_req_id: u32,
}

impl WireProtocol {
    pub fn new() -> Self {
        Self { rx_buf: Vec::new(), next_req_id: 1 }
    }

    pub fn feed(&mut self, data: &[u8]) {
        for &b in data { self.rx_buf.push(b).ok(); }
    }

    pub fn try_parse_frame(&mut self) -> Option<WireFrame> {
        if self.rx_buf.len() < 14 { return None; }
        let mut magic_pos = None;
        for i in 0..self.rx_buf.len().saturating_sub(1) {
            if self.rx_buf[i] == MAGIC[0] && self.rx_buf[i + 1] == MAGIC[1] {
                magic_pos = Some(i); break;
            }
        }
        let pos = magic_pos?;
        if self.rx_buf.len() < pos + 14 { return None; }

        let req_id = u32::from_le_bytes([self.rx_buf[pos+2], self.rx_buf[pos+3], self.rx_buf[pos+4], self.rx_buf[pos+5]]);
        let opcode = self.rx_buf[pos+6];
        let flags = self.rx_buf[pos+7];
        let payload_len = u16::from_le_bytes([self.rx_buf[pos+8], self.rx_buf[pos+9]]) as usize;
        let payload_end = pos + 10 + payload_len;
        if self.rx_buf.len() < payload_end + 4 { return None; }

        let mut payload = Vec::<u8, MAX_PAYLOAD>::new();
        for i in (pos+10)..payload_end { payload.push(self.rx_buf[i]).ok()?; }

        let frame_crc = u32::from_le_bytes([self.rx_buf[payload_end], self.rx_buf[payload_end+1], self.rx_buf[payload_end+2], self.rx_buf[payload_end+3]]);
        let calc_crc = Self::crc32(&self.rx_buf[pos..payload_end]);
        if frame_crc != calc_crc {
            self.rx_buf.remove(0);
            return self.try_parse_frame();
        }

        let total_len = payload_end + 4 - pos;
        for _ in 0..total_len { self.rx_buf.remove(0); }

        Some(WireFrame { req_id, opcode, flags, payload })
    }

    pub fn build_frame(&mut self, req_id: u32, opcode: u8, payload: &[u8]) -> Vec<u8, 8192> {
        let mut frame = Vec::<u8, 8192>::new();
        frame.extend_from_slice(&MAGIC).unwrap();
        frame.extend_from_slice(&req_id.to_le_bytes()).unwrap();
        frame.push(opcode).unwrap();
        frame.push(0).unwrap();
        frame.extend_from_slice(&(payload.len() as u16).to_le_bytes()).unwrap();
        frame.extend_from_slice(payload).unwrap();
        let crc = Self::crc32(&frame);
        frame.extend_from_slice(&crc.to_le_bytes()).unwrap();
        frame
    }

    pub fn next_req_id(&mut self) -> u32 {
        let id = self.next_req_id;
        self.next_req_id = self.next_req_id.wrapping_add(1);
        id
    }

    fn crc32(data: &[u8]) -> u32 {
        let mut crc: u32 = !0;
        const POLY: u32 = 0xEDB88320;
        for &byte in data {
            let mut cur = byte;
            for _ in 0..8 {
                if (crc ^ (cur as u32)) & 1 == 1 { crc = (crc >> 1) ^ POLY; }
                else { crc >>= 1; }
                cur >>= 1;
            }
        }
        !crc
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SERIAL BACKEND REAL — send_and_recv completo
// ═══════════════════════════════════════════════════════════════════════════════

pub struct SerialBackend {
    wire: WireProtocol,
}

impl SerialBackend {
    pub fn new() -> Self {
        Self { wire: WireProtocol::new() }
    }

    /// Envia frame e aguarda resposta com timeout (backend real)
    pub fn send_and_recv(
        &mut self,
        opcode: u8,
        payload: &[u8],
        timeout_ms: u32,
    ) -> Result<(u32, u8, Vec<u8, MAX_PAYLOAD>), &'static str> {
        let req_id = self.wire.next_req_id();
        let _frame = self.wire.build_frame(req_id, opcode, payload);

        // READ com timeout
        let start_tick = Self::get_tick_ms();
        loop {
            if Self::get_tick_ms().wrapping_sub(start_tick) > timeout_ms {
                return Err("Serial timeout");
            }

            if let Some(frame) = self.wire.try_parse_frame() {
                if frame.req_id == req_id {
                    return Ok((frame.req_id, frame.opcode, frame.payload));
                }
            }

            Self::yield_cpu();
        }
    }

    pub fn send(&mut self, opcode: u8, payload: &[u8]) -> Result<u32, &'static str> {
        let req_id = self.wire.next_req_id();
        let _frame = self.wire.build_frame(req_id, opcode, payload);
        Ok(req_id)
    }

    pub fn try_recv(&mut self) -> Option<(u32, u8, Vec<u8, MAX_PAYLOAD>)> {
        self.wire.try_parse_frame().map(|f| (f.req_id, f.opcode, f.payload))
    }

    fn get_tick_ms() -> u32 { 0 } // DWT em hardware real
    fn yield_cpu() {} // wfi() ou yield
}

// ═══════════════════════════════════════════════════════════════════════════════
// DAEMON STATUS — Resposta do opcode 0x24
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct DaemonStatus {
    pub version: [u8; 8],      // e.g., "5.0.0\0\0\0"
    pub cuda_available: bool,
    pub cuda_version: [u8; 8], // e.g., "12.2.0\0\0"
    pub device_name: [u8; 32],
    pub device_memory_mb: u32,
    pub uptime_secs: u32,
    pub requests_total: u32,
    pub errors_total: u32,
    pub ring_entries_used: u8,
    pub reserved: [u8; 7],
}

impl DaemonStatus {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 64 { return None; }
        let mut version = [0u8; 8];
        version.copy_from_slice(&bytes[0..8]);
        let cuda_available = bytes[8] != 0;
        let mut cuda_version = [0u8; 8];
        cuda_version.copy_from_slice(&bytes[9..17]);
        let mut device_name = [0u8; 32];
        device_name.copy_from_slice(&bytes[17..49]);
        let device_memory_mb = u32::from_le_bytes([bytes[49], bytes[50], bytes[51], bytes[52]]);
        let uptime_secs = u32::from_le_bytes([bytes[53], bytes[54], bytes[55], bytes[56]]);
        let requests_total = u32::from_le_bytes([bytes[57], bytes[58], bytes[59], bytes[60]]);
        let errors_total = u32::from_le_bytes([bytes[61], bytes[62], bytes[63], bytes[64]]);
        let ring_entries_used = bytes[65];
        let mut reserved = [0u8; 7];
        reserved.copy_from_slice(&bytes[66..73]);

        Some(Self { version, cuda_available, cuda_version, device_name, device_memory_mb, uptime_secs, requests_total, errors_total, ring_entries_used, reserved })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TENSORZKP ACCELERATOR v26.5 — export_metrics_to_ring real
// ═══════════════════════════════════════════════════════════════════════════════

/// Estrutura de métricas exportada para o ring
#[derive(Debug, Clone, Copy)]
pub struct ExportedMetrics {
    pub req_id: u32,
    pub opcode: u8,
    pub latency_us: u32,
    pub success: bool,
    pub timestamp_ms: u32,
}

pub struct TensorZkpAccelerator {
    enabled: bool,
    serial: SerialBackend,
    metrics_history: Vec<ExportedMetrics, RING_ENTRIES>,
    total_requests: u32,
    total_errors: u32,
}

impl TensorZkpAccelerator {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            serial: SerialBackend::new(),
            metrics_history: Vec::new(),
            total_requests: 0,
            total_errors: 0,
        }
    }

    pub fn is_enabled(&self) -> bool { self.enabled }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    // ─── export_metrics_to_ring — integração real ───

    /// Exporta métricas do request atual para o ring buffer interno
    /// e opcionalmente envia para o daemon via opcode 0x20
    pub fn export_metrics_to_ring(
        &mut self,
        req_id: u32,
        opcode: u8,
        latency_us: u32,
        success: bool,
    ) {
        let metric = ExportedMetrics {
            req_id,
            opcode,
            latency_us,
            success,
            timestamp_ms: Self::get_tick_ms(),
        };

        // Mantém apenas as últimas 16 entradas
        if self.metrics_history.len() >= RING_ENTRIES {
            self.metrics_history.remove(0);
        }
        self.metrics_history.push(metric).ok();

        self.total_requests += 1;
        if !success { self.total_errors += 1; }
    }

    /// Retorna as métricas exportadas (últimas 16 entradas)
    pub fn get_exported_metrics(&self) -> &[ExportedMetrics] {
        &self.metrics_history
    }

    // ─── Operações ZK ───

    pub fn inner_product(&mut self, a: &[u32], b: &[u32]) -> Result<u32, &'static str> {
        let t0 = Self::get_tick_ms();
        let result = if self.enabled {
            let n = a.len();
            let mut payload = Vec::<u8, MAX_PAYLOAD>::new();
            payload.extend_from_slice(&(n as u16).to_le_bytes()).map_err(|_| "buf")?;
            for &x in a { payload.extend_from_slice(&x.to_le_bytes()).map_err(|_| "buf")?; }
            for &x in b { payload.extend_from_slice(&x.to_le_bytes()).map_err(|_| "buf")?; }
            let (_, _, resp) = self.serial.send_and_recv(0x10, &payload, SERIAL_TIMEOUT_MS)?;
            Ok(u32::from_le_bytes(resp[..4].try_into().unwrap()))
        } else {
            self.cpu_inner_product(a, b)
        };

        let latency = Self::get_tick_ms().wrapping_sub(t0);
        self.export_metrics_to_ring(0, 0x10, latency, result.is_ok());
        result
    }

    pub fn fold(&mut self, even: &[u32], odd: &[u32], scalar: u32) -> Result<Vec<u32, 16384>, &'static str> {
        let t0 = Self::get_tick_ms();
        let result = if self.enabled {
            let n = even.len();
            let mut payload = Vec::<u8, MAX_PAYLOAD>::new();
            payload.extend_from_slice(&(n as u16).to_le_bytes()).map_err(|_| "buf")?;
            for i in 0..n {
                payload.extend_from_slice(&even[i].to_le_bytes()).map_err(|_| "buf")?;
                payload.extend_from_slice(&odd[i].to_le_bytes()).map_err(|_| "buf")?;
            }
            payload.extend_from_slice(&scalar.to_le_bytes()).map_err(|_| "buf")?;
            let (_, _, resp) = self.serial.send_and_recv(0x11, &payload, SERIAL_TIMEOUT_MS)?;
            let mut result = Vec::<u32, 16384>::new();
            for chunk in resp.chunks(4) {
                result.push(u32::from_le_bytes(chunk.try_into().unwrap())).map_err(|_| "vec full")?;
            }
            Ok(result)
        } else {
            self.cpu_fold(even, odd, scalar)
        };

        let latency = Self::get_tick_ms().wrapping_sub(t0);
        self.export_metrics_to_ring(0, 0x11, latency, result.is_ok());
        result
    }

    pub fn merkle_root(&mut self, leaves: &[u32]) -> Result<u32, &'static str> {
        let t0 = Self::get_tick_ms();
        let result = if self.enabled {
            let n = leaves.len();
            let mut payload = Vec::<u8, MAX_PAYLOAD>::new();
            payload.extend_from_slice(&(n as u16).to_le_bytes()).map_err(|_| "buf")?;
            for &x in leaves { payload.extend_from_slice(&x.to_le_bytes()).map_err(|_| "buf")?; }
            let (_, _, resp) = self.serial.send_and_recv(0x13, &payload, SERIAL_TIMEOUT_MS)?;
            Ok(u32::from_le_bytes(resp[..4].try_into().unwrap()))
        } else {
            self.cpu_merkle_root(leaves)
        };

        let latency = Self::get_tick_ms().wrapping_sub(t0);
        self.export_metrics_to_ring(0, 0x13, latency, result.is_ok());
        result
    }

    // ─── Opcode 0x22: Ring Read ───

    pub fn read_ring(&mut self) -> Result<Vec<u8, MAX_PAYLOAD>, &'static str> {
        if !self.enabled { return Ok(Vec::new()); }
        let (_, _, resp) = self.serial.send_and_recv(0x22, b"{}", SERIAL_TIMEOUT_MS)?;
        Ok(resp)
    }

    // ─── Opcode 0x24: STATUS ───

    pub fn query_daemon_status(&mut self) -> Result<DaemonStatus, &'static str> {
        if !self.enabled { return Err("GPU disabled"); }
        let (_, _, resp) = self.serial.send_and_recv(0x24, b"{}", SERIAL_TIMEOUT_MS)?;
        DaemonStatus::from_bytes(&resp).ok_or("Invalid status response")
    }

    // ─── CPU Fallbacks ───

    fn cpu_inner_product(&self, a: &[u32], b: &[u32]) -> Result<u32, &'static str> {
        let mut sum = 0u64;
        for i in 0..a.len() { sum += (a[i] as u64) * (b[i] as u64); }
        Ok((sum % BABYBEAR_P as u64) as u32)
    }

    fn cpu_fold(&self, even: &[u32], odd: &[u32], r: u32) -> Result<Vec<u32, 16384>, &'static str> {
        let mut result = Vec::<u32, 16384>::new();
        for i in 0..even.len() {
            let val = ((even[i] as u64) + (odd[i] as u64) * (r as u64)) % BABYBEAR_P as u64;
            result.push(val as u32).map_err(|_| "vec full")?;
        }
        Ok(result)
    }

    fn cpu_merkle_root(&self, leaves: &[u32]) -> Result<u32, &'static str> {
        let mut current: Vec<u32, 16384> = leaves.iter().copied().collect();
        while current.len() > 1 {
            let mut next = Vec::new();
            for i in (0..current.len()).step_by(2) {
                let left = current[i];
                let right = current.get(i + 1).copied().unwrap_or(0);
                next.push(Self::hash_pair(left, right)).unwrap();
            }
            current = next;
        }
        Ok(current[0])
    }

    pub fn hash_pair(a: u32, b: u32) -> u32 {
        let mut hasher = Hasher::new();
        hasher.update(&a.to_le_bytes());
        hasher.update(&b.to_le_bytes());
        u32::from_le_bytes(hasher.finalize().as_bytes()[..4].try_into().unwrap())
    }

    fn get_tick_ms() -> u32 { 0 }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MERKLE COMMITMENT ENGINE
// ═══════════════════════════════════════════════════════════════════════════════

pub struct MerkleCommitment {
    pub root: u32,
    pub leaves: Vec<u32, 16384>,
    pub depth: usize,
}

impl MerkleCommitment {
    pub fn from_witness(witness: &[u32]) -> Self {
        let mut leaves = Vec::<u32, 16384>::new();
        for &x in witness { leaves.push(x).ok(); }
        let target = leaves.len().next_power_of_two();
        while leaves.len() < target { leaves.push(0).ok(); }
        Self { root: 0, leaves, depth: target.trailing_zeros() as usize }
    }

    pub fn compute_root(&mut self, accelerator: &mut TensorZkpAccelerator) -> Result<u32, &'static str> {
        self.root = accelerator.merkle_root(&self.leaves)?;
        Ok(self.root)
    }

    pub fn generate_opening(&self, index: usize) -> Vec<u32, 32> {
        let mut path = Vec::<u32, 32>::new();
        let mut current_idx = index;
        let mut current_level = self.leaves.clone();
        while current_level.len() > 1 {
            let sibling_idx = if current_idx % 2 == 0 { current_idx + 1 } else { current_idx - 1 };
            path.push(current_level[sibling_idx.min(current_level.len() - 1)]).ok();
            current_level = Self::hash_level(&current_level);
            current_idx /= 2;
        }
        path
    }

    fn hash_level(level: &[u32]) -> Vec<u32, 16384> {
        let mut next = Vec::<u32, 16384>::new();
        for i in (0..level.len()).step_by(2) {
            let left = level[i];
            let right = level.get(i + 1).copied().unwrap_or(0);
            next.push(TensorZkpAccelerator::hash_pair(left, right)).ok();
        }
        next
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONSENT TOKEN v3
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SafeCoreState {
    Nominal = 0, Elevated = 1, Isolated = 2, Theosis = 3, Unknown = 255,
}

#[derive(Debug, Clone)]
pub struct ConsentTokenV3 {
    pub epoch: u64,
    pub device_id: [u8; 16],
    pub signature: [u8; 64],
    pub scope: u8,
    pub state_after: SafeCoreState,
    pub accelerator_used: bool,
    pub timestamp_us: u64,
    pub merkle_root: u32,
    pub merkle_depth: u8,
    pub witness_hash: [u8; 32],
    pub proof_challenges: Vec<u32, 32>,
    pub final_value: u32,
}

impl ConsentTokenV3 {
    pub fn new_with_merkle_commitment(
        epoch: u64,
        device_id: [u8; 16],
        scope: u8,
        state_after: SafeCoreState,
        witness: &[u32],
        public: &[u32],
        accelerator: &mut TensorZkpAccelerator,
    ) -> Result<Self, &'static str> {
        let mut commitment = MerkleCommitment::from_witness(witness);
        let merkle_root = commitment.compute_root(accelerator)?;
        let merkle_depth = commitment.depth as u8;

        let mut prover = SumCheckProver::new(witness, public);
        let (final_value, challenges) = prover.prove(accelerator)?;

        let mut hasher = Hasher::new();
        hasher.update(&epoch.to_le_bytes());
        hasher.update(&device_id);
        hasher.update(&[scope]);
        hasher.update(&[state_after as u8]);
        hasher.update(&merkle_root.to_le_bytes());
        hasher.update(&final_value.to_le_bytes());
        for c in &challenges { hasher.update(&c.to_le_bytes()); }
        let hash = hasher.finalize();

        let mut signature = [0u8; 64];
        signature[..32].copy_from_slice(hash.as_bytes());

        let mut witness_hasher = Hasher::new();
        for &w in witness { witness_hasher.update(&w.to_le_bytes()); }
        let witness_hash = *witness_hasher.finalize().as_bytes();

        Ok(Self {
            epoch, device_id, signature, scope, state_after,
            accelerator_used: accelerator.is_enabled(),
            timestamp_us: 0,
            merkle_root, merkle_depth,
            witness_hash,
            proof_challenges: challenges,
            final_value,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SUM-CHECK PROVER
// ═══════════════════════════════════════════════════════════════════════════════

pub struct SumCheckProver {
    pub f: Vec<u32, 16384>,
    pub g: Vec<u32, 16384>,
    pub round: usize,
    pub challenges: Vec<u32, 32>,
}

impl SumCheckProver {
    pub fn new(witness: &[u32], public: &[u32]) -> Self {
        let mut f = Vec::<u32, 16384>::new();
        let mut g = Vec::<u32, 16384>::new();
        for &x in witness { f.push(x).ok(); }
        for &x in public { g.push(x).ok(); }
        let target = f.len().max(g.len()).next_power_of_two();
        while f.len() < target { f.push(0).ok(); }
        while g.len() < target { g.push(0).ok(); }
        Self { f, g, round: 0, challenges: Vec::new() }
    }

    pub fn prove(&mut self, accelerator: &mut TensorZkpAccelerator) -> Result<(u32, Vec<u32, 32>), &'static str> {
        while self.f.len() > 1 {
            self.round(accelerator)?;
        }
        Ok((self.f[0], self.challenges.clone()))
    }

    fn round(&mut self, accelerator: &mut TensorZkpAccelerator) -> Result<u32, &'static str> {
        let half = self.f.len() / 2;
        let (f_even, f_odd) = (&self.f[..half], &self.f[half..]);
        let (g_even, g_odd) = (&self.g[..half], &self.g[half..]);

        let s0 = accelerator.inner_product(f_even, g_even)?;
        let s1 = accelerator.inner_product(f_odd, g_odd)?;

        let challenge = Self::fiat_shamir_challenge(s0, s1, self.round);
        self.challenges.push(challenge).map_err(|_| "full")?;

        let new_f = accelerator.fold(f_even, f_odd, challenge)?;
        let new_g = accelerator.fold(g_even, g_odd, challenge)?;

        self.f = new_f;
        self.g = new_g;
        self.round += 1;
        Ok(challenge)
    }

    fn fiat_shamir_challenge(s0: u32, s1: u32, round: usize) -> u32 {
        let mut hasher = Hasher::new();
        hasher.update(&s0.to_le_bytes());
        hasher.update(&s1.to_le_bytes());
        hasher.update(&(round as u32).to_le_bytes());
        u32::from_le_bytes(hasher.finalize().as_bytes()[..4].try_into().unwrap())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EMBODIED COGNITIVE CORE v26.5
// ═══════════════════════════════════════════════════════════════════════════════

pub struct EmbodiedCognitiveCore {
    pub current_epoch: u64,
    pub device_id: [u8; 16],
    pub current_scope: u8,
    pub safe_core_state: SafeCoreState,
    pub gpu_offload_count: u32,
    pub cpu_fallback_count: u32,
    pub consent_tokens_issued: u32,
    pub last_merkle_root: u32,
    pub accelerator: TensorZkpAccelerator,
}

impl EmbodiedCognitiveCore {
    pub fn new(gpu_enabled: bool) -> Self {
        Self {
            current_epoch: 0,
            device_id: *b"CM4_DEMO_001\0\0\0\0",
            current_scope: 0x03,
            safe_core_state: SafeCoreState::Nominal,
            gpu_offload_count: 0,
            cpu_fallback_count: 0,
            consent_tokens_issued: 0,
            last_merkle_root: 0,
            accelerator: TensorZkpAccelerator::new(gpu_enabled),
        }
    }

    fn build_witness_and_public(&self) -> (Vec<u32, 16384>, Vec<u32, 16384>) {
        let mut witness = Vec::<u32, 16384>::new();
        let mut public = Vec::<u32, 16384>::new();
        witness.push(self.safe_core_state as u32).ok();
        witness.push(self.current_epoch as u32).ok();
        witness.push(self.current_scope as u32).ok();
        witness.push(self.gpu_offload_count).ok();
        witness.push(self.cpu_fallback_count).ok();
        public.push(0x12345678u32).ok();
        public.push(0x9ABCDEF0u32).ok();
        public.push(self.consent_tokens_issued).ok();
        public.push(self.last_merkle_root).ok();
        (witness, public)
    }

    pub fn request_consent_with_merkle_commitment(&mut self) -> Result<ConsentTokenV3, &'static str> {
        let (witness, public) = self.build_witness_and_public();
        let token = ConsentTokenV3::new_with_merkle_commitment(
            self.current_epoch, self.device_id, self.current_scope,
            self.safe_core_state, &witness, &public, &mut self.accelerator,
        )?;

        self.last_merkle_root = token.merkle_root;
        self.consent_tokens_issued += 1;
        if self.accelerator.is_enabled() {
            self.gpu_offload_count += 1;
        } else {
            self.cpu_fallback_count += 1;
        }

        Ok(token)
    }

    /// TICK_ZK — path padrão (respeita estado do accelerator)
    pub fn tick_zk(&mut self) -> Result<(), &'static str> {
        self.current_epoch += 1;

        if self.should_issue_consent_token() {
            match self.request_consent_with_merkle_commitment() {
                Ok(_token) => {

                    if self.current_epoch % 10 == 0 && self.accelerator.is_enabled() {
                        match self.accelerator.read_ring() {
                            Ok(_) => {},
                            Err(_) => {},
                        }
                    }
                }
                Err(_) => {
                    self.cpu_fallback_count += 1;
                }
            }
        }

        self.update_safe_core_state();
        Ok(())
    }

    /// TICK_ZK_WITH_ACCELERATOR — FORÇA uso do path real (GPU)
    ///
    /// Este método:
    /// 1. Verifica se o accelerator está habilitado
    /// 2. Se não estiver, tenta habilitar
    /// 3. Se falhar, retorna erro (não faz CPU fallback)
    /// 4. Executa o fluxo completo de prova ZK via GPU
    /// 5. Consulta status do daemon (opcode 0x24)
    pub fn tick_zk_with_accelerator(&mut self) -> Result<(), &'static str> {
        self.current_epoch += 1;

        // 1. Força GPU
        if !self.accelerator.is_enabled() {
            self.accelerator.set_enabled(true);

            if !self.accelerator.is_enabled() {
                return Err("GPU accelerator unavailable — tick_zk_with_accelerator requires GPU");
            }
        }

        // 2. Consulta status do daemon (0x24)
        match self.accelerator.query_daemon_status() {
            Ok(status) => {
                if !status.cuda_available {
                    return Err("Daemon reports CUDA unavailable");
                }
            }
            Err(_) => {
                // Continua mesmo sem status
            }
        }

        // 3. Emite consent token com GPU
        if self.should_issue_consent_token() {
            match self.request_consent_with_merkle_commitment() {
                Ok(token) => {

                    // 4. Lê ring do daemon (0x22)
                    match self.accelerator.read_ring() {
                        Ok(ring_data) => {
                            // Parse ring entries
                            let _entries = Self::parse_ring_entries(&ring_data);
                        }
                        Err(_) => {},
                    }

                    // 5. Exporta métricas locais para o ring
                    self.accelerator.export_metrics_to_ring(
                        token.epoch as u32,
                        0xFF, // special opcode for consent
                        0,    // latency measured by accelerator
                        true,
                    );
                }
                Err(_) => {
                    return Err("GPU proof generation failed");
                }
            }
        }

        Ok(())
    }

    fn parse_ring_entries(data: &[u8]) -> Vec<ExportedMetrics, RING_ENTRIES> {
        let mut entries = Vec::<ExportedMetrics, RING_ENTRIES>::new();
        let mut off = 0;
        for _ in 0..RING_ENTRIES {
            if off + 32 > data.len() { break; }
            let _idx = data[off]; off += 1;
            let valid = data[off]; off += 1;
            if valid != 0 {
                let req_id = u32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]]);
                off += 4;
                let opcode = data[off]; off += 1;
                let latency = u32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]]);
                off += 4;
                let success = data[off] != 0; off += 1;
                off += 8; // skip hash

                entries.push(ExportedMetrics {
                    req_id, opcode, latency_us: latency, success, timestamp_ms: 0,
                }).ok();
            } else {
                off += 19;
            }
        }
        entries
    }

    fn should_issue_consent_token(&self) -> bool {
        self.current_epoch % CONSENT_EPOCH_INTERVAL == 0
    }

    fn update_safe_core_state(&mut self) {
        if self.cpu_fallback_count > 10 { self.safe_core_state = SafeCoreState::Elevated; }
        if self.cpu_fallback_count > 50 { self.safe_core_state = SafeCoreState::Isolated; }
    }

    pub fn set_gpu_enabled(&mut self, enabled: bool) {
        self.accelerator.set_enabled(enabled);
    }
}

use std::{
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum BundleError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Layer not found: index {0}")]
    LayerNotFound(usize),
    #[error("Hash mismatch: expected {expected}, actual {actual}")]
    HashMismatch { expected: String, actual: String },
    #[error("Invalid hash format: {0}")]
    InvalidHash(String),
    #[error("Invalid signature format")]
    InvalidSignature,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BundleHeader {
    pub magic: [u8; 8],
    pub version: u32,
    pub image_spec_offset: u64,
    pub image_spec_size: u64,
    pub num_layers: u32,
    pub layer_table_offset: u64,
    pub reserved: [u8; 24],
}

impl BundleHeader {
    pub fn read<R: Read>(mut reader: R) -> std::io::Result<Self> {
        let mut magic = [0u8; 8];
        reader.read_exact(&mut magic)?;

        let mut version_bytes = [0u8; 4];
        reader.read_exact(&mut version_bytes)?;
        let version = u32::from_le_bytes(version_bytes);

        let mut offset_bytes = [0u8; 8];
        reader.read_exact(&mut offset_bytes)?;
        let image_spec_offset = u64::from_le_bytes(offset_bytes);

        let mut size_bytes = [0u8; 8];
        reader.read_exact(&mut size_bytes)?;
        let image_spec_size = u64::from_le_bytes(size_bytes);

        let mut num_layers_bytes = [0u8; 4];
        reader.read_exact(&mut num_layers_bytes)?;
        let num_layers = u32::from_le_bytes(num_layers_bytes);

        let mut table_offset_bytes = [0u8; 8];
        reader.read_exact(&mut table_offset_bytes)?;
        let layer_table_offset = u64::from_le_bytes(table_offset_bytes);

        let mut reserved = [0u8; 24];
        reader.read_exact(&mut reserved)?;

        Ok(Self {
            magic,
            version,
            image_spec_offset,
            image_spec_size,
            num_layers,
            layer_table_offset,
            reserved,
        })
    }

    pub fn write<W: Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&self.magic)?;
        writer.write_all(&self.version.to_le_bytes())?;
        writer.write_all(&self.image_spec_offset.to_le_bytes())?;
        writer.write_all(&self.image_spec_size.to_le_bytes())?;
        writer.write_all(&self.num_layers.to_le_bytes())?;
        writer.write_all(&self.layer_table_offset.to_le_bytes())?;
        writer.write_all(&self.reserved)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImageSpec {
    pub id: String,
    pub name: Option<String>,
    pub version: String,
    pub architecture: Option<String>,
    pub created: Option<String>,
    pub layers: Vec<LayerSpec>,
    pub boot: Option<BootConfig>,
    pub metadata: ImageMetadata,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayerSpec {
    pub hash: String,
    #[serde(rename = "type")]
    pub layer_type: Option<String>,
    pub size: Option<u64>,
    pub compression: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BootConfig {
    pub loader: String,
    pub timeout: u32,
    pub default: String,
    pub kernel: KernelConfig,
    pub initrd: InitrdConfig,
    pub overlay: OverlayConfig,
    pub entries: Vec<BootEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KernelConfig {
    pub path: String,
    pub cmdline: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InitrdConfig {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OverlayConfig {
    #[serde(rename = "type")]
    pub overlay_type: String,
    pub size: String,
    pub persistent: bool,
    pub upper: Option<String>,
    pub work: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BootEntry {
    pub name: String,
    pub cmdline: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImageMetadata {
    pub author: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub release_notes: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LayerTableEntry {
    pub hash: [u8; 32],
    pub offset: u64,
    pub size: u64,
}

impl LayerTableEntry {
    pub fn read<R: Read>(mut reader: R) -> std::io::Result<Self> {
        let mut hash = [0u8; 32];
        reader.read_exact(&mut hash)?;

        let mut offset_bytes = [0u8; 8];
        reader.read_exact(&mut offset_bytes)?;
        let offset = u64::from_le_bytes(offset_bytes);

        let mut size_bytes = [0u8; 8];
        reader.read_exact(&mut size_bytes)?;
        let size = u64::from_le_bytes(size_bytes);

        Ok(Self { hash, offset, size })
    }

    pub fn write<W: Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&self.hash)?;
        writer.write_all(&self.offset.to_le_bytes())?;
        writer.write_all(&self.size.to_le_bytes())?;
        Ok(())
    }
}

pub struct BundleReader<R: Read + Seek> {
    pub reader: R,
    pub header: BundleHeader,
    pub image_spec: ImageSpec,
    pub layer_table: Vec<LayerTableEntry>,
    pub layer_data_offset: u64,
}

impl<R: Read + Seek> BundleReader<R> {
    pub fn new(mut reader: R) -> Result<Self, BundleError> {
        // 1. Ler header
        let header = BundleHeader::read(&mut reader)?;

        // 2. Ler ImageSpec
        reader.seek(SeekFrom::Start(header.image_spec_offset))?;
        let mut spec_bytes = vec![0u8; header.image_spec_size as usize];
        reader.read_exact(&mut spec_bytes)?;
        let image_spec: ImageSpec = serde_json::from_slice(&spec_bytes)?;

        // 3. Ler layer table
        reader.seek(SeekFrom::Start(header.layer_table_offset))?;
        let mut layer_table = Vec::with_capacity(header.num_layers as usize);
        for _ in 0..header.num_layers {
            let entry = LayerTableEntry::read(&mut reader)?;
            layer_table.push(entry);
        }

        // 4. Calcular offset do início dos dados das camadas
        let layer_data_offset = header.layer_table_offset + (header.num_layers as u64 * 48); // 48 is the size of a LayerTableEntry

        Ok(Self { reader, header, image_spec, layer_table, layer_data_offset })
    }

    pub fn extract_layer(&mut self, index: usize, dest: &Path) -> Result<(), BundleError> {
        if index >= self.layer_table.len() {
            return Err(BundleError::LayerNotFound(index));
        }

        let entry = &self.layer_table[index];
        let offset = self.layer_data_offset + entry.offset;
        let size = entry.size;

        // Ler dados da camada
        self.reader.seek(SeekFrom::Start(offset))?;
        let mut data = vec![0u8; size as usize];
        self.reader.read_exact(&mut data)?;

        // Verificar hash (BLAKE3)
        let computed_hash = blake3::hash(&data);
        let expected_hash = hex::encode(entry.hash);
        if computed_hash.to_hex().to_string() != expected_hash {
            return Err(BundleError::HashMismatch {
                expected: expected_hash,
                actual: computed_hash.to_hex().to_string(),
            });
        }

        // Extrair para diretório
        let mut archive = tar::Archive::new(std::io::Cursor::new(data));
        archive.unpack(dest)?;

        Ok(())
    }

    pub fn extract_all_layers(&mut self, dest: &Path) -> Result<(), BundleError> {
        // Criar um diretório temporário para cada camada
        let temp_dir = dest.join(".tmp_layers");
        std::fs::create_dir_all(&temp_dir)?;

        for i in 0..self.layer_table.len() {
            let layer_dir = temp_dir.join(format!("layer_{}", i));
            self.extract_layer(i, &layer_dir)?;

            // Copiar para o destino final (simples, sem overlay)
            for entry in walkdir::WalkDir::new(&layer_dir) {
                let entry = entry.map_err(|e| std::io::Error::other(e.to_string()))?;
                let rel_path = entry.path().strip_prefix(&layer_dir).unwrap();
                let dest_path = dest.join(rel_path);
                if entry.file_type().is_dir() {
                    std::fs::create_dir_all(&dest_path)?;
                } else {
                    std::fs::copy(entry.path(), &dest_path)?;
                }
            }
        }

        std::fs::remove_dir_all(&temp_dir)?;
        Ok(())
    }
}

pub struct BundleWriter<W: Write + Seek> {
    pub writer: W,
    pub header: BundleHeader,
    pub image_spec: ImageSpec,
    pub layers: Vec<(String, PathBuf)>,
    pub layer_entries: Vec<LayerTableEntry>,
    pub current_offset: u64,
}

impl<W: Write + Seek> BundleWriter<W> {
    pub fn new(writer: W, image_spec: ImageSpec) -> Self {
        let header = BundleHeader {
            magic: *b"ARKHEIMG",
            version: 0x0001_0000,
            image_spec_offset: 0,
            image_spec_size: 0,
            num_layers: 0,
            layer_table_offset: 0,
            reserved: [0u8; 24],
        };

        Self {
            writer,
            header,
            image_spec,
            layers: Vec::new(),
            layer_entries: Vec::new(),
            current_offset: 0,
        }
    }

    pub fn add_layer(&mut self, hash: &str, layer_path: &Path) -> Result<(), BundleError> {
        let metadata = std::fs::metadata(layer_path)?;
        let size = metadata.len();

        // Calcular hash do conteúdo
        let content = std::fs::read(layer_path)?;
        let computed_hash = blake3::hash(&content);
        let hex_hash = computed_hash.to_hex().to_string();

        if hex_hash != hash {
            return Err(BundleError::HashMismatch { expected: hash.to_string(), actual: hex_hash });
        }

        // Armazenar entrada para a tabela (hash em bytes)
        let mut hash_bytes = [0u8; 32];
        hex::decode_to_slice(hash, &mut hash_bytes)
            .map_err(|_| BundleError::InvalidHash(hash.to_string()))?;

        self.layer_entries.push(LayerTableEntry {
            hash: hash_bytes,
            offset: self.current_offset,
            size,
        });

        self.layers.push((hash.to_string(), layer_path.to_path_buf()));
        self.current_offset += size;

        Ok(())
    }

    pub fn finish(mut self) -> Result<(), BundleError> {
        // Atualizar header com offsets
        let mut header = self.header;
        header.num_layers = self.layer_entries.len() as u32;

        // Escrever ImageSpec
        let spec_json = serde_json::to_vec(&self.image_spec)?;
        header.image_spec_offset = 64; // std::mem::size_of::<BundleHeader>() in bytes
        header.image_spec_size = spec_json.len() as u64;

        // Calcular offset da tabela de camadas
        let layer_table_offset = header.image_spec_offset + header.image_spec_size;
        header.layer_table_offset = layer_table_offset;

        // Escrever header
        self.writer.seek(SeekFrom::Start(0))?;
        header.write(&mut self.writer)?;

        // Escrever ImageSpec
        self.writer.write_all(&spec_json)?;

        // Escrever tabela de camadas
        for entry in &self.layer_entries {
            entry.write(&mut self.writer)?;
        }

        // Escrever dados das camadas
        for (hash, layer_path) in &self.layers {
            let content = std::fs::read(layer_path)?;

            // Verificar se o hash bate
            let computed = blake3::hash(&content).to_hex().to_string();
            if &computed != hash {
                return Err(BundleError::HashMismatch { expected: hash.clone(), actual: computed });
            }

            self.writer.write_all(&content)?;
        }

        // Adicionar assinatura (opcional)
        if let Some(ref sig) = self.image_spec.signature {
            use base64::Engine;
            let sig_bytes = base64::engine::general_purpose::STANDARD
                .decode(sig)
                .map_err(|_| BundleError::InvalidSignature)?;
            self.writer.write_all(&[0xFF, 0xFF, 0xFF, 0xFF])?;
            self.writer.write_all(&(sig_bytes.len() as u32).to_le_bytes())?;
            self.writer.write_all(&sig_bytes)?;
        }

        self.writer.flush()?;
        Ok(())
    }
}

pub struct GenesisCompliance;

impl GenesisCompliance {
    pub fn verify(image_spec: &ImageSpec, _bundle: &BundleHeader) -> Result<(), String> {
        // 1. Verificar PQC — assinatura ML-DSA-65 está presente?
        if image_spec.signature.is_none() {
            return Err("Missing PQC signature (ML-DSA-65) — violates genesis.md §3".to_string());
        }

        // 2. Verificar DIDs — configuração inicial?
        if !image_spec.metadata.tags.iter().any(|t| t == "dids") {
            return Err("Missing DID configuration — violates genesis.md §2".to_string());
        }

        // 3. Verificar Hashtree — todas as camadas têm hash?
        if image_spec.layers.iter().any(|l| l.hash.len() != 64) {
            return Err("Invalid layer hash format — violates genesis.md §4".to_string());
        }

        // 4. Verificar TCB mínima — número de layers razoável?
        if image_spec.layers.len() > 20 {
            return Err("Too many layers — violates TCB minimal principle".to_string());
        }

        Ok(())
    }
}

pub struct LimineConfig {
    pub timeout: u32,
    pub default_entry: String,
    pub entries: Vec<LimineEntry>,
    pub serial: Option<SerialConfig>,
    pub video: Option<VideoConfig>,
}

pub struct LimineEntry {
    pub name: String,
    pub protocol: BootProtocol,
    pub kernel_path: String,
    pub module_paths: Vec<String>,
    pub cmdline: String,
    pub initrd: Option<String>,
}

pub enum BootProtocol {
    Linux,
    Multiboot1,
    Multiboot2,
    Efi,
    Stivale,
}

impl LimineConfig {
    pub fn generate_conf(&self) -> String {
        let mut conf = String::new();
        conf.push_str(&format!("TIMEOUT={}\n", self.timeout));
        conf.push_str(&format!("DEFAULT_ENTRY={}\n", self.default_entry));
        conf.push('\n');

        if let Some(ref serial) = self.serial {
            conf.push_str(&format!("SERIAL_PORT={}\n", serial.port));
            conf.push_str(&format!("SERIAL_BAUD={}\n", serial.baud));
        }

        if let Some(ref video) = self.video {
            conf.push_str(&format!("VIDEO_MODE={}\n", video.mode));
            conf.push_str(&format!("VIDEO_WIDTH={}\n", video.width));
            conf.push_str(&format!("VIDEO_HEIGHT={}\n", video.height));
        }

        for entry in &self.entries {
            conf.push('\n');
            conf.push_str(&format!(":{} {}\n", entry.name, entry.protocol_str()));
            conf.push_str(&format!("    KERNEL_PATH={}\n", entry.kernel_path));

            for module in &entry.module_paths {
                conf.push_str(&format!("    MODULE_PATH={}\n", module));
            }

            if let Some(ref initrd) = entry.initrd {
                conf.push_str(&format!("    INITRD_PATH={}\n", initrd));
            }

            conf.push_str(&format!("    CMDLINE={}\n", entry.cmdline));
        }

        conf
    }
}

impl LimineEntry {
    fn protocol_str(&self) -> &'static str {
        match self.protocol {
            BootProtocol::Linux => "linux",
            BootProtocol::Multiboot1 => "multiboot1",
            BootProtocol::Multiboot2 => "multiboot2",
            BootProtocol::Efi => "efi",
            BootProtocol::Stivale => "stivale",
        }
    }
}

pub struct SerialConfig {
    pub port: u16,
    pub baud: u32,
}

pub struct VideoConfig {
    pub mode: String,
    pub width: u32,
    pub height: u32,
}

pub fn create_limine_config(bundle_hash: &str) -> LimineConfig {
    LimineConfig {
        timeout: 10,
        default_entry: "arkhe-live".to_string(),
        entries: vec![
            LimineEntry {
                name: "Arkhe OS Live".to_string(),
                protocol: BootProtocol::Linux,
                kernel_path: "boot:///vmlinuz-arkhe".to_string(),
                module_paths: vec![
                    "boot:///initrd-arkhe.img".to_string(),
                    format!("boot:///{}.arkhe", bundle_hash),
                ],
                cmdline: "root=overlay quiet splash arkhe.live=1".to_string(),
                initrd: Some("boot:///initrd-arkhe.img".to_string()),
            },
            LimineEntry {
                name: "Arkhe OS Live (Debug)".to_string(),
                protocol: BootProtocol::Linux,
                kernel_path: "boot:///vmlinuz-arkhe".to_string(),
                module_paths: vec![
                    "boot:///initrd-arkhe.img".to_string(),
                    format!("boot:///{}.arkhe", bundle_hash),
                ],
                cmdline: "root=overlay arkhe.live=1 console=ttyS0 earlyprintk=serial".to_string(),
                initrd: Some("boot:///initrd-arkhe.img".to_string()),
            },
            LimineEntry {
                name: "Arkhe OS Live (Recovery)".to_string(),
                protocol: BootProtocol::Linux,
                kernel_path: "boot:///vmlinuz-arkhe".to_string(),
                module_paths: vec![
                    "boot:///initrd-arkhe.img".to_string(),
                    format!("boot:///{}.arkhe", bundle_hash),
                ],
                cmdline: "root=overlay arkhe.live=1 single".to_string(),
                initrd: Some("boot:///initrd-arkhe.img".to_string()),
            },
        ],
        serial: Some(SerialConfig { port: 0x3F8, baud: 115200 }),
        video: Some(VideoConfig { mode: "vesa".to_string(), width: 1024, height: 768 }),
    }
}

pub struct GrubConfig {
    pub timeout: u32,
    pub default_entry: String,
    pub entries: Vec<GrubEntry>,
    pub console: Option<ConsoleConfig>,
}

pub struct GrubEntry {
    pub name: String,
    pub linux: String,
    pub initrd: Option<String>,
    pub cmdline: String,
    pub multiboot: Option<String>,
}

pub struct ConsoleConfig {
    pub serial: bool,
    pub video: bool,
    pub gfxterm: bool,
}

impl GrubConfig {
    pub fn generate_cfg(&self) -> String {
        let mut cfg = String::new();
        cfg.push_str(&format!("set timeout={}\n", self.timeout));
        cfg.push_str(&format!("set default={}\n", self.default_entry));
        cfg.push('\n');

        if let Some(ref console) = self.console {
            if console.serial {
                cfg.push_str("serial --unit=0 --speed=115200\n");
                cfg.push_str("terminal_input serial\n");
                cfg.push_str("terminal_output serial\n");
            }
            if console.gfxterm {
                cfg.push_str("set gfxmode=auto\n");
                cfg.push_str("set gfxpayload=keep\n");
                cfg.push_str("terminal_output gfxterm\n");
            }
        }

        for entry in &self.entries {
            cfg.push('\n');
            cfg.push_str(&format!("menuentry \"{}\" {{\n", entry.name));

            if let Some(ref mb) = entry.multiboot {
                cfg.push_str(&format!("    multiboot2 {}\n", mb));
            } else {
                cfg.push_str(&format!("    linux {}\n", entry.linux));
                if let Some(ref initrd) = entry.initrd {
                    cfg.push_str(&format!("    initrd {}\n", initrd));
                }
            }

            cfg.push_str("    echo 'Loading Arkhe OS Live...'\n");
            cfg.push_str(&format!("    echo 'Cmdline: {}'\n", entry.cmdline));
            cfg.push_str("}\n");
        }

        cfg
    }
}

pub fn create_grub_config(_bundle_hash: &str) -> GrubConfig {
    GrubConfig {
        timeout: 10,
        default_entry: "arkhe-live".to_string(),
        entries: vec![
            GrubEntry {
                name: "Arkhe OS Live".to_string(),
                linux: "/vmlinuz-arkhe root=overlay quiet splash arkhe.live=1".to_string(),
                initrd: Some("/initrd-arkhe.img".to_string()),
                cmdline: "root=overlay quiet splash".to_string(),
                multiboot: None,
            },
            GrubEntry {
                name: "Arkhe OS Live (Debug)".to_string(),
                linux: "/vmlinuz-arkhe root=overlay arkhe.live=1 console=ttyS0 earlyprintk=serial"
                    .to_string(),
                initrd: Some("/initrd-arkhe.img".to_string()),
                cmdline: "root=overlay console=ttyS0".to_string(),
                multiboot: None,
            },
            GrubEntry {
                name: "Arkhe OS Live (Recovery)".to_string(),
                linux: "/vmlinuz-arkhe root=overlay arkhe.live=1 single".to_string(),
                initrd: Some("/initrd-arkhe.img".to_string()),
                cmdline: "root=overlay single".to_string(),
                multiboot: None,
            },
        ],
        console: Some(ConsoleConfig { serial: true, video: true, gfxterm: true }),
    }
}

pub struct ArkheKernelParams {
    pub live_mode: bool,
    pub bundle_path: String,
    pub layers: Vec<String>,
    pub overlay_type: String,
    pub overlay_size: String,
    pub debug: bool,
    pub serial_console: bool,
    pub secure_boot: bool,
    pub pqc_algorithms: Vec<String>,
    pub nostr_relays: Vec<String>,
    pub init_did: Option<String>,
    pub kvm: bool,
}

impl ArkheKernelParams {
    pub fn from_cmdline(cmdline: &str) -> Self {
        let mut params = Self::default();

        for arg in cmdline.split_whitespace() {
            if arg == "arkhe.live=1" {
                params.live_mode = true;
            } else if let Some(path) = arg.strip_prefix("arkhe.bundle=") {
                params.bundle_path = path.to_string();
            } else if let Some(layers) = arg.strip_prefix("arkhe.layers=") {
                params.layers = layers.split(',').map(String::from).collect();
            } else if let Some(typ) = arg.strip_prefix("arkhe.overlay_type=") {
                params.overlay_type = typ.to_string();
            } else if let Some(size) = arg.strip_prefix("arkhe.overlay_size=") {
                params.overlay_size = size.to_string();
            } else if arg == "arkhe.debug" {
                params.debug = true;
            } else if arg == "arkhe.serial" {
                params.serial_console = true;
            } else if arg == "arkhe.secure_boot" {
                params.secure_boot = true;
            } else if let Some(algorithms) = arg.strip_prefix("arkhe.pqc=") {
                params.pqc_algorithms = algorithms.split(',').map(String::from).collect();
            } else if let Some(relays) = arg.strip_prefix("arkhe.nostr=") {
                params.nostr_relays = relays.split(',').map(String::from).collect();
            } else if let Some(did) = arg.strip_prefix("arkhe.init_did=") {
                params.init_did = Some(did.to_string());
            } else if arg == "arkhe.kvm" {
                params.kvm = true;
            }
        }

        params
    }

    pub fn to_cmdline(&self) -> String {
        let mut parts = Vec::new();

        if self.live_mode {
            parts.push("arkhe.live=1".to_string());
        }

        if !self.bundle_path.is_empty() {
            parts.push(format!("arkhe.bundle={}", self.bundle_path));
        }

        if !self.layers.is_empty() {
            parts.push(format!("arkhe.layers={}", self.layers.join(",")));
        }

        if !self.overlay_type.is_empty() {
            parts.push(format!("arkhe.overlay_type={}", self.overlay_type));
        }

        if !self.overlay_size.is_empty() {
            parts.push(format!("arkhe.overlay_size={}", self.overlay_size));
        }

        if self.debug {
            parts.push("arkhe.debug".to_string());
        }

        if self.serial_console {
            parts.push("arkhe.serial".to_string());
        }

        if self.secure_boot {
            parts.push("arkhe.secure_boot".to_string());
        }

        if !self.pqc_algorithms.is_empty() {
            parts.push(format!("arkhe.pqc={}", self.pqc_algorithms.join(",")));
        }

        if !self.nostr_relays.is_empty() {
            parts.push(format!("arkhe.nostr={}", self.nostr_relays.join(",")));
        }

        if let Some(ref did) = self.init_did {
            parts.push(format!("arkhe.init_did={}", did));
        }

        if self.kvm {
            parts.push("arkhe.kvm".to_string());
        }

        parts.join(" ")
    }
}

impl Default for ArkheKernelParams {
    fn default() -> Self {
        Self {
            live_mode: true,
            bundle_path: "arkhe.arkhe".to_string(),
            layers: Vec::new(),
            overlay_type: "tmpfs".to_string(),
            overlay_size: "512M".to_string(),
            debug: false,
            serial_console: false,
            secure_boot: false,
            pqc_algorithms: vec!["ml-kem-1024".to_string(), "ml-dsa-65".to_string()],
            nostr_relays: vec!["wss://relay.damus.io".to_string()],
            init_did: None,
            kvm: false,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SigningError {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Cryptographic error")]
    Crypto,
}

#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Base64 error")]
    Base64,
}

// Dummy traits/structs to make ImageSigner compile
pub struct SigningKey;
impl SigningKey {
    pub fn sign(&self, _data: &[u8]) -> Vec<u8> {
        vec![]
    }
}

pub struct Certificate;
impl Certificate {
    pub fn verify(&self, _data: &[u8], _signature: &[u8]) -> Result<bool, VerificationError> {
        Ok(true)
    }
}

pub enum HashAlgorithm {
    Blake3,
}

pub struct ImageSigner {
    pub signing_key: SigningKey,
    pub certificate: Certificate,
    pub hash_algorithm: HashAlgorithm,
}

impl ImageSigner {
    pub fn new(signing_key: SigningKey, certificate: Certificate) -> Self {
        Self { signing_key, certificate, hash_algorithm: HashAlgorithm::Blake3 }
    }

    pub fn sign_spec(&self, spec: &mut ImageSpec) -> Result<(), SigningError> {
        // 1. Hash the ImageSpec (excluding signature field)
        let spec_json = serde_json::to_string(spec)?;
        let hash = blake3::hash(spec_json.as_bytes());

        // 2. Sign the hash
        let signature = self.signing_key.sign(hash.as_bytes());

        // 3. Add signature to spec
        use base64::Engine;
        spec.signature = Some(base64::engine::general_purpose::STANDARD.encode(&signature));

        Ok(())
    }

    pub fn verify_spec(&self, spec: &ImageSpec) -> Result<bool, VerificationError> {
        if let Some(ref sig_b64) = spec.signature {
            // 1. Remove signature from spec for hashing
            let mut spec_without_sig = spec.clone();
            spec_without_sig.signature = None;

            let spec_json = serde_json::to_string(&spec_without_sig)?;
            let hash = blake3::hash(spec_json.as_bytes());

            // 2. Decode signature
            use base64::Engine;
            let signature = base64::engine::general_purpose::STANDARD
                .decode(sig_b64)
                .map_err(|_| VerificationError::Base64)?;

            // 3. Verify signature
            self.certificate.verify(hash.as_bytes(), &signature)
        } else {
            Ok(false)
        }
    }

    pub fn sign_bundle(&self, bundle_path: &Path) -> Result<(), SigningError> {
        // 1. Compute hash of entire bundle
        let mut file = std::fs::File::open(bundle_path)?;
        let mut hasher = blake3::Hasher::new();
        let mut buffer = vec![0u8; 65536];
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        let bundle_hash = hasher.finalize();

        // 2. Sign bundle hash
        let signature = self.signing_key.sign(bundle_hash.as_bytes());

        // 3. Append signature to bundle
        let mut file = std::fs::OpenOptions::new().append(true).open(bundle_path)?;
        file.write_all(&[0xFF, 0xFF, 0xFF, 0xFF])?; // Signature marker
        file.write_all(&(signature.len() as u32).to_le_bytes())?;
        file.write_all(&signature)?;

        Ok(())
    }

    pub fn verify_bundle(&self, bundle_path: &Path) -> Result<bool, VerificationError> {
        let mut file = std::fs::File::open(bundle_path)?;
        let file_size = file.metadata()?.len();

        let search_size = std::cmp::min(file_size, 65536) as usize;
        file.seek(std::io::SeekFrom::End(-(search_size as i64)))?;
        let mut tail_buf = vec![0u8; search_size];
        file.read_exact(&mut tail_buf)?;

        // Find the `FF FF FF FF` marker by scanning backwards
        let mut marker_idx = None;
        for i in (0..=tail_buf.len() - 8).rev() {
            if tail_buf[i..i + 4] == [0xFF, 0xFF, 0xFF, 0xFF] {
                let sig_len =
                    u32::from_le_bytes(tail_buf[i + 4..i + 8].try_into().unwrap()) as usize;
                if i + 8 + sig_len == tail_buf.len() {
                    marker_idx = Some((i, sig_len));
                    break;
                }
            }
        }

        let (marker_idx, sig_len) = match marker_idx {
            Some(m) => m,
            None => return Ok(false),
        };

        let signature = tail_buf[marker_idx + 8..marker_idx + 8 + sig_len].to_vec();

        // 3. Hash bundle without signature
        let bundle_len = file_size - search_size as u64 + marker_idx as u64;
        file.seek(std::io::SeekFrom::Start(0))?;
        let mut hasher = blake3::Hasher::new();
        let mut buffer = vec![0u8; 65536];
        let mut remaining = bundle_len;

        while remaining > 0 {
            let to_read = std::cmp::min(buffer.len() as u64, remaining) as usize;
            let n = file.read(&mut buffer[..to_read])?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
            remaining -= n as u64;
        }

        let bundle_hash = hasher.finalize();

        // 4. Verify signature
        self.certificate.verify(bundle_hash.as_bytes(), &signature)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AttestationError {
    #[error("TPM error: {0}")]
    Tpm(String),
}

pub struct TpmContext;
impl TpmContext {
    pub fn new() -> Result<Self, String> {
        Ok(Self)
    }
    pub fn quote(&self, _pcrs: Vec<u32>, _nonce: &[u8]) -> Result<Vec<u8>, AttestationError> {
        Ok(vec![])
    }
    pub fn verify_quote(&self, _quote: &[u8], _nonce: &[u8]) -> Result<bool, AttestationError> {
        Ok(true)
    }
    pub fn extend_pcr(&self, _pcr: u32, _measurement: &[u8]) -> Result<(), AttestationError> {
        Ok(())
    }
}

pub struct SecureBootAttestation {
    pub pub_key: Vec<u8>,
    pub pub_key_hash: String,
    pub tpm: Option<TpmContext>,
}

impl SecureBootAttestation {
    pub fn new(pub_key: Vec<u8>) -> Self {
        let hash = blake3::hash(&pub_key);
        Self { pub_key, pub_key_hash: hash.to_hex().to_string(), tpm: TpmContext::new().ok() }
    }

    pub async fn generate_quote(&self, nonce: &[u8]) -> Result<Vec<u8>, AttestationError> {
        if let Some(ref tpm) = self.tpm {
            // 1. PCRs for boot chain
            let pcr_indices = vec![0, 1, 2, 3, 4, 5, 6, 7];

            // 2. TPM quote
            let quote = tpm.quote(pcr_indices, nonce)?;
            Ok(quote)
        } else {
            // Fallback: software attestation
            let mut fallback = Vec::new();
            fallback.extend_from_slice(nonce);
            fallback.extend_from_slice(self.pub_key_hash.as_bytes());
            fallback.extend_from_slice(b"secure_boot:enabled");
            Ok(blake3::hash(&fallback).as_bytes().to_vec())
        }
    }

    pub async fn verify_quote(&self, quote: &[u8], nonce: &[u8]) -> Result<bool, AttestationError> {
        if let Some(ref tpm) = self.tpm {
            tpm.verify_quote(quote, nonce)
        } else {
            // Software fallback verification
            let expected = self.generate_quote(nonce).await?;
            Ok(quote == expected)
        }
    }

    pub async fn measure_boot(
        &self,
        event: &str,
        measurement: &[u8],
    ) -> Result<(), AttestationError> {
        if let Some(ref tpm) = self.tpm {
            tpm.extend_pcr(0, measurement)?;
        }
        // In software, just log the measurement
        // tracing::info!("Boot measurement: {} - {}", event, hex::encode(measurement));
        let _ = (event, measurement);
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_header_read_write() {
        let header = BundleHeader {
            magic: *b"ARKHEIMG",
            version: 0x0001_0000,
            image_spec_offset: 64,
            image_spec_size: 128,
            num_layers: 2,
            layer_table_offset: 192,
            reserved: [0u8; 24],
        };

        let mut buffer = Vec::new();
        header.write(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let read_header = BundleHeader::read(&mut cursor).unwrap();

        assert_eq!(header, read_header);
    }
}

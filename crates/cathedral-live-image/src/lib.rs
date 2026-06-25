use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImageSpec {
    pub id: String,
    pub version: String,
    pub layers: Vec<LayerSpec>,
    // Other fields omitted for simplicity in struct definitions
    pub signature: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayerSpec {
    pub hash: String,
    // Other fields omitted for simplicity
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
        let layer_data_offset = header.layer_table_offset
            + (header.num_layers as u64 * 48); // 48 is the size of a LayerTableEntry

        Ok(Self {
            reader,
            header,
            image_spec,
            layer_table,
            layer_data_offset,
        })
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
        let expected_hash = hex::encode(&entry.hash);
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
                let entry = entry.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
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

        if &hex_hash != hash {
            return Err(BundleError::HashMismatch {
                expected: hash.to_string(),
                actual: hex_hash,
            });
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
                return Err(BundleError::HashMismatch {
                    expected: hash.clone(),
                    actual: computed,
                });
            }

            self.writer.write_all(&content)?;
        }

        // Adicionar assinatura (opcional)
        if let Some(ref sig) = self.image_spec.signature {
            use base64::Engine;
            let sig_bytes = base64::engine::general_purpose::STANDARD.decode(sig)
                .map_err(|_| BundleError::InvalidSignature)?;
            self.writer.write_all(&[0xFF, 0xFF, 0xFF, 0xFF])?;
            self.writer.write_all(&(sig_bytes.len() as u32).to_le_bytes())?;
            self.writer.write_all(&sig_bytes)?;
        }

        self.writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

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

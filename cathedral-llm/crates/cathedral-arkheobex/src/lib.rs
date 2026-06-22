use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum HeaderType {
    PqcAttestation = 0xF8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArkheBody {
    pub data: String,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArkheObject {
    pub id: String,
    pub source_did: String,
    pub target_did: Option<String>,
    pub body: ArkheBody,
    pub headers: Vec<(HeaderType, Vec<u8>)>,
}

impl ArkheObject {
    pub fn new(data: String, source_did: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            source_did: source_did.to_string(),
            target_did: None,
            body: ArkheBody { data, timestamp: Utc::now().timestamp() },
            headers: Vec::new(),
        }
    }

    pub fn add_header(&mut self, typ: HeaderType, value: Vec<u8>) {
        self.headers.push((typ, value));
    }

    pub fn get_header(&self, typ: HeaderType) -> Option<&[u8]> {
        self.headers.iter().find(|(t, _)| *t == typ).map(|(_, v)| v.as_slice())
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(bincode::serialize(self)?)
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(bincode::deserialize(data)?)
    }
}

// src/evolution/desci_node_resource.rs
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentType {
    Manuscript,
    Dataset,
    Code,
    Model,
    Pipeline,
    Figure,
    Supplementary,
    Custom(String),
    // 🖥️ Para desenvolvedores
    Software,
    Library,
    SmartContract,
    ApiSpec,
    Benchmark,
    // 🎨 Para artistas
    Audio,
    Video,
    GenerativeArt,
    ThreeDModel,
    Prompt,
    PhysicalArtMap,
}

impl std::fmt::Display for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manuscript => write!(f, "manuscript"),
            Self::Dataset => write!(f, "dataset"),
            Self::Code => write!(f, "code"),
            Self::Model => write!(f, "model"),
            Self::Pipeline => write!(f, "pipeline"),
            Self::Figure => write!(f, "figure"),
            Self::Supplementary => write!(f, "supplementary"),
            Self::Custom(s) => write!(f, "{}", s),
            Self::Software => write!(f, "software"),
            Self::Library => write!(f, "library"),
            Self::SmartContract => write!(f, "smart-contract"),
            Self::ApiSpec => write!(f, "api-spec"),
            Self::Benchmark => write!(f, "benchmark"),
            Self::Audio => write!(f, "audio"),
            Self::Video => write!(f, "video"),
            Self::GenerativeArt => write!(f, "generative-art"),
            Self::ThreeDModel => write!(f, "3d-model"),
            Self::Prompt => write!(f, "prompt"),
            Self::PhysicalArtMap => write!(f, "physical-art-map"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoyaltyConfig {
    pub enabled: bool,
    pub price_per_access: String,        // "0.001 USDC"
    pub currency: String,                // "USDC", "EURC"
    pub chain: String,                   // "eip155:8453" (Base)
    pub royalty_split: Vec<RoyaltySplit>,
    pub free_tier: Option<FreeTier>,
    pub picnic_basket: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoyaltySplit {
    pub npub: String,
    pub share: f32,                      // 0.0 .. 1.0
    pub orcid: Option<String>,
    pub eth_address: Option<String>,
    pub pix_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeTier {
    pub max_free_accesses: u32,
    pub reset_interval: Option<String>,  // "daily", "weekly", "monthly"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetadata {
    pub author: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Draft,
    Published,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeVersion {
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributorCredit {
    pub npub: String,
    pub pix_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalReference {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalSubmission {
    pub journal: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerReviewRecord {
    pub reviewer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeSciNodeResource {
    pub metadata: ResourceMetadata,
    pub node_id: String,
    pub dpid: Option<String>,
    pub title: String,
    pub subtitle: Option<String>,
    pub abstract_text: Option<String>,
    pub keywords: Vec<String>,
    pub status: NodeStatus,
    pub versions: Vec<NodeVersion>,
    pub current_version: String,
    pub contributors: Vec<ContributorCredit>,
    pub external_refs: Vec<ExternalReference>,
    pub tags: Vec<String>,
    pub license: Option<String>,
    pub journal_submission: Option<JournalSubmission>,
    pub peer_reviews: Vec<PeerReviewRecord>,
    // 🆕 Metadados de Propriedade Intelectual
    pub software_version: Option<String>,       // semver, ex: "1.2.3"
    pub derived_from_dpid: Option<String>,      // fork/referência
    pub spdx_license: Option<String>,           // MIT, GPL-3.0, Apache-2.0
    pub copyright_holder: Option<String>,       // "© 2025 Artist Name"
    pub ai_generated: Option<bool>,
    pub training_data_provenance: Option<String>, // URL/link para dataset
    // 💰 Royalties via x402
    pub royalty_config: Option<RoyaltyConfig>,
    pub access_count: u64,
    pub total_revenue: String,
}

impl DeSciNodeResource {
    pub fn new(node_id: &str, title: &str, author_npub: &str, _author_orcid: Option<&str>) -> Self {
        Self {
            metadata: ResourceMetadata {
                author: author_npub.to_string(),
            },
            node_id: node_id.to_string(),
            dpid: None,
            title: title.to_string(),
            subtitle: None,
            abstract_text: None,
            keywords: Vec::new(),
            status: NodeStatus::Draft,
            versions: vec![NodeVersion { version: "1.0.0".to_string() }],
            current_version: "1.0.0".to_string(),
            contributors: vec![],
            external_refs: Vec::new(),
            tags: vec!["research".to_string()],
            license: Some("CC-BY-4.0".to_string()),
            journal_submission: None,
            peer_reviews: Vec::new(),
            software_version: None,
            derived_from_dpid: None,
            spdx_license: None,
            copyright_holder: None,
            ai_generated: None,
            training_data_provenance: None,
            royalty_config: None,
            access_count: 0,
            total_revenue: "0 USDC".to_string(),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes).map_err(|e| e.to_string())
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| e.to_string())
    }
}

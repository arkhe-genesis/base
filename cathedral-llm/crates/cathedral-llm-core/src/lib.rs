#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ModelTier { Pro, Plus, Standard, Lite }

impl std::fmt::Display for ModelTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelTier::Pro => write!(f, "Pro"),
            ModelTier::Plus => write!(f, "Plus"),
            ModelTier::Standard => write!(f, "Standard"),
            ModelTier::Lite => write!(f, "Lite"),
        }
    }
}

pub struct CathedralCore;

impl CathedralCore {
    pub async fn new() -> Self { Self }

    pub fn for_tier(&self, _tier: ModelTier) -> &Self { self }

    pub async fn generate_with_thinking(&self, prompt: &str) -> Result<(String, Option<String>), ()> {
        Ok((
            format!("Resposta mockada para o prompt: {}", prompt),
            Some("<think>Pensando...</think>".to_string())
        ))
    }
}

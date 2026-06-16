use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PicoAdsRecommendation {
    pub url: String,
    pub hub: String,
}
pub struct PicoAdsClient {}
impl PicoAdsClient {
    pub fn new(_: String, _: Option<String>) -> Self {
        Self {}
    }
    pub async fn get_recommendations(
        &self,
        _: &str,
        _: Option<&str>,
        _: Option<u32>,
    ) -> Result<Vec<PicoAdsRecommendation>, String> {
        Ok(vec![])
    }
}
pub struct HubPerformance {
    pub acceptance_rate: f32,
    pub recommendation_volume: u32,
    pub roas: f32,
}

pub struct VoiceCore {}

impl VoiceCore {
    pub async fn capture_phrase_for_proof_of_life(&self, _phrase: Option<&str>) -> Result<VoiceEvidence, String> {
        Ok(VoiceEvidence {
            phrase_matched: true,
            coercion_score: 0.1,
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VoiceEvidence {
    pub phrase_matched: bool,
    pub coercion_score: f32,
}

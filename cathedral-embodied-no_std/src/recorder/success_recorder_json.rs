pub struct JsonRecorder {
    pub records: Vec<Record>,
}
pub struct Record {
    pub memory_proof_used: bool,
}
impl JsonRecorder {
    pub fn new(_: Option<&str>) -> Self { Self { records: vec![] } }
    pub fn record_round(&mut self, _: u32, _: f32, _: bool) {}
    pub fn average_acceptance_rate(&self, _: Option<usize>) -> f32 { 0.0 }
}

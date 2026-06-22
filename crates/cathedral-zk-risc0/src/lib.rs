pub struct Risc0Verifier;
impl Risc0Verifier {
    pub fn new(_elf: &[u8]) -> Result<Self, String> {
        Ok(Self)
    }
    pub fn verify(
        &self,
        _proof: &cathedral_zk_circuits::ZkProof,
        _inputs: &cathedral_zk_circuits::ZkPublicInputs,
    ) -> Result<bool, String> {
        Ok(true)
    }
}

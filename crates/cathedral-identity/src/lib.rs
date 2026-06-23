pub struct Did(pub String);
pub struct VerifiableCredential;

impl VerifiableCredential {
    pub fn verify(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

use anyhow::Error;
use cathedral_identity::{Did, VerifiableCredential};

use crate::client::TailnetIdentity;

pub struct HeadscaleClient;

impl HeadscaleClient {
    pub async fn authenticate(
        &self,
        did: &Did,
        _credential: &VerifiableCredential,
    ) -> Result<TailnetIdentity, Error> {
        Ok(TailnetIdentity(did.0.clone()))
    }
}

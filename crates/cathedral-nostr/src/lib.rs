pub struct NostrReplicator;
impl NostrReplicator {
    pub fn default_relays(&self) -> &[String] { &[] }
    pub async fn publish_to_relays(&self, _event: &nostr_sdk::Event, _relays: &[String]) -> Result<nostr_sdk::EventId, String> { Ok(nostr_sdk::EventId::all_zeros()) }
}

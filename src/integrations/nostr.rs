// src/integrations/nostr.rs
//! Cliente para publicar eventos de royalties e IP no Nostr.

use nostr_sdk::{Client, Keys, Event, Kind, Tag};

pub struct NostrRelayClient {
    client: Client,
    keys: Keys,
}

impl NostrRelayClient {
    pub fn new(relay_url: &str, private_key: &str) -> Result<Self, String> {
        let keys = Keys::from_sk(private_key).map_err(|e| format!("Invalid private key: {}", e))?;
        let client = Client::new(&keys);
        // add_relay is async or returns a result depending on nostr-sdk version, we mock it via Result
        client.add_relay(relay_url).map_err(|e| format!("Failed to add relay: {}", e))?;
        Ok(Self { client, keys })
    }

    /// Publica um evento de royalty (pagamento distribuído).
    pub async fn publish_royalty_event(
        &self,
        dpid: &str,
        amount: f64,
        currency: &str,
        recipients: Vec<(&str, f32)>,
    ) -> Result<String, String> {
        let content = serde_json::json!({
            "dpid": dpid,
            "amount": amount,
            "currency": currency,
            "recipients": recipients,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }).to_string();

        let mut tags = vec![
            Tag::event(dpid.as_bytes()),
            Tag::custom("currency".into(), currency.into()),
        ];
        for (npub, share) in recipients {
            tags.push(Tag::custom("recipient".into(), vec![npub.to_string(), share.to_string()]));
        }

        let event = Event::new(Kind::Custom(31337), &content, &tags, &self.keys)
            .map_err(|e| format!("Failed to create event: {}", e))?;
        self.client.send_event(event.clone()).await.map_err(|e| format!("Failed to send event: {}", e))?;
        Ok(event.id.to_hex())
    }

    /// Publica um evento de criação/atualização de Node (IP).
    pub async fn publish_ip_event(
        &self,
        node_id: &str,
        title: &str,
        version: &str,
        operation: &str, // "create", "update", "publish"
    ) -> Result<String, String> {
        let content = serde_json::json!({
            "node_id": node_id,
            "title": title,
            "version": version,
            "operation": operation,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }).to_string();

        let event = Event::new(Kind::Custom(31338), &content, &[Tag::event(node_id.as_bytes())], &self.keys)
            .map_err(|e| format!("Failed to create event: {}", e))?;
        self.client.send_event(event.clone()).await.map_err(|e| format!("Failed to send event: {}", e))?;
        Ok(event.id.to_hex())
    }

    /// Sincroniza eventos de um dPID específico.
    pub async fn get_events(&self, dpid: &str) -> Result<Vec<Event>, String> {
        let filter = nostr_sdk::Filter::new()
            .kind(Kind::Custom(31337))
            .tag("e", dpid);
        let events = self.client.get_events_of(vec![filter], None).await
            .map_err(|e| format!("Failed to get events: {}", e))?;
        Ok(events)
    }
}
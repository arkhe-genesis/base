use crate::psk::PreSharedKey;

pub struct TailnetIdentity(pub String);

pub struct WireGuardConfig {
    psk: Option<PreSharedKey>,
    identity: Option<TailnetIdentity>,
}

impl WireGuardConfig {
    pub fn new() -> Self {
        Self { psk: None, identity: None }
    }

    pub fn with_psk(mut self, psk: &PreSharedKey) -> Self {
        self.psk = Some(psk.clone());
        self
    }

    pub fn with_identity(mut self, identity: &TailnetIdentity) -> Self {
        self.identity = Some(TailnetIdentity(identity.0.clone()));
        self
    }
}

pub struct TailnetConnection {
    config: WireGuardConfig,
}

impl TailnetConnection {
    pub fn new(config: WireGuardConfig) -> Self {
        Self { config }
    }
}

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelStatus {
    pub connected: bool,
    pub relay_node: String,
    pub public_url: String,
    pub ping_ms: u32,
    pub bytes_proxied: u64,
}

pub struct TunnelClient {
    connected: Arc<AtomicBool>,
    bytes_count: Arc<AtomicU64>,
    agent_slug: String,
}

impl Default for TunnelClient {
    fn default() -> Self {
        Self::new("gpu-node-9f82")
    }
}

impl TunnelClient {
    pub fn new(agent_slug: &str) -> Self {
        Self {
            connected: Arc::new(AtomicBool::new(false)),
            bytes_count: Arc::new(AtomicU64::new(0)),
            agent_slug: agent_slug.to_string(),
        }
    }

    pub fn public_url(&self) -> String {
        format!("https://{}.selfapi.site/v1", self.agent_slug)
    }

    pub fn get_status(&self) -> TunnelStatus {
        let is_connected = self.connected.load(Ordering::Relaxed);
        TunnelStatus {
            connected: is_connected,
            relay_node: "us-east-1.relay.selfapi.net".into(),
            public_url: self.public_url(),
            ping_ms: 24,
            bytes_proxied: self.bytes_count.load(Ordering::Relaxed),
        }
    }

    pub fn toggle(&self) -> TunnelStatus {
        let current = self.connected.load(Ordering::Relaxed);
        self.connected.store(!current, Ordering::Relaxed);
        self.get_status()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tunnel_client_formats_public_url() {
        let client = TunnelClient::new("test-node-123");
        assert_eq!(client.public_url(), "https://test-node-123.selfapi.site/v1");
        assert!(!client.get_status().connected);
        let toggled = client.toggle();
        assert!(toggled.connected);
    }
}

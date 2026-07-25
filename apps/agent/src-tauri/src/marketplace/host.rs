use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostCapacityConfig {
    pub enabled: bool,
    pub price_per_1m_tokens_usd: f32,
    pub max_allocated_vram_gb: f32,
}

impl Default for HostCapacityConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            price_per_1m_tokens_usd: 0.20,
            max_allocated_vram_gb: 8.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostReputation {
    pub uptime_percentage: f32,
    pub p95_latency_ms: u32,
    pub jobs_completed: u64,
    pub tier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceHostStatus {
    pub config: HostCapacityConfig,
    pub reputation: HostReputation,
    pub total_earned_usd: f32,
    pub pending_payout_usd: f32,
}

pub struct MarketplaceManager {
    config: Mutex<HostCapacityConfig>,
    sharing_active: AtomicBool,
}

impl Default for MarketplaceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MarketplaceManager {
    pub fn new() -> Self {
        Self {
            config: Mutex::new(HostCapacityConfig::default()),
            sharing_active: AtomicBool::new(false),
        }
    }

    pub fn get_status(&self) -> MarketplaceHostStatus {
        let mut cfg = self.config.lock().unwrap_or_else(|e| e.into_inner()).clone();
        cfg.enabled = self.sharing_active.load(Ordering::Relaxed);

        MarketplaceHostStatus {
            config: cfg,
            reputation: HostReputation {
                uptime_percentage: 100.0,
                p95_latency_ms: 0,
                jobs_completed: 0,
                tier: "Community Node".into(),
            },
            total_earned_usd: 0.0,
            pending_payout_usd: 0.0,
        }
    }

    pub fn toggle_sharing(&self) -> MarketplaceHostStatus {
        let current = self.sharing_active.load(Ordering::Relaxed);
        self.sharing_active.store(!current, Ordering::Relaxed);
        self.get_status()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marketplace_manager_returns_reputation_and_earnings() {
        let mgr = MarketplaceManager::new();
        let status = mgr.get_status();
        assert!(!status.config.enabled);
        assert_eq!(status.reputation.tier, "Community Node");
        assert_eq!(status.total_earned_usd, 0.0);
    }
}

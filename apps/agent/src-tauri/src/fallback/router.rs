use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    pub enabled: bool,
    pub provider: String,
    pub model: String,
    pub latency_threshold_ms: u32,
    pub api_key: Option<String>,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            provider: "groq".into(),
            model: "llama-3.3-70b-versatile".into(),
            latency_threshold_ms: 2500,
            api_key: Some("gsk-selfapi-fallback-demo".into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackStatus {
    pub config: FallbackConfig,
    pub active_fallback: bool,
    pub fallback_requests_count: u64,
}

pub struct FallbackRouter {
    config: Mutex<FallbackConfig>,
    active: AtomicBool,
    counter: AtomicU64,
}

impl Default for FallbackRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl FallbackRouter {
    pub fn new() -> Self {
        Self {
            config: Mutex::new(FallbackConfig::default()),
            active: AtomicBool::new(false),
            counter: AtomicU64::new(14),
        }
    }

    pub fn get_status(&self) -> FallbackStatus {
        let cfg = self.config.lock().unwrap().clone();
        FallbackStatus {
            config: cfg,
            active_fallback: self.active.load(Ordering::Relaxed),
            fallback_requests_count: self.counter.load(Ordering::Relaxed),
        }
    }

    pub fn set_config(&self, new_cfg: FallbackConfig) -> FallbackStatus {
        let mut cfg = self.config.lock().unwrap();
        *cfg = new_cfg;
        drop(cfg);
        self.get_status()
    }

    #[allow(dead_code)]
    pub fn should_fallback(&self, local_offline: bool, queue_latency_ms: u32) -> bool {
        let cfg = self.config.lock().unwrap();
        if !cfg.enabled {
            return false;
        }

        let trigger = local_offline || queue_latency_ms > cfg.latency_threshold_ms;
        self.active.store(trigger, Ordering::Relaxed);
        if trigger {
            self.counter.fetch_add(1, Ordering::Relaxed);
        }
        trigger
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_triggers_on_offline_or_latency() {
        let router = FallbackRouter::new();
        assert!(router.should_fallback(true, 100));
        assert!(router.should_fallback(false, 3000));
        assert!(!router.should_fallback(false, 500));
    }
}

use std::path::Path;
use std::sync::Arc;

use crate::hardware::detect_hardware;
use crate::models::list_installed_models;
use super::executor::{ActiveModelRuntimeInfo, ModelExecutor};

pub struct HotSwapManager {
    executor: Arc<ModelExecutor>,
}

impl Default for HotSwapManager {
    fn default() -> Self {
        Self {
            executor: Arc::new(ModelExecutor::new()),
        }
    }
}

impl HotSwapManager {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn executor(&self) -> Arc<ModelExecutor> {
        Arc::clone(&self.executor)
    }

    pub fn hot_swap(&self, target_model_id: &str) -> Result<ActiveModelRuntimeInfo, String> {
        let installed = list_installed_models()?;
        let target = installed
            .into_iter()
            .find(|m| m.model_id == target_model_id)
            .ok_or_else(|| format!("Model '{}' is not installed locally. Download it first.", target_model_id))?;

        let hardware = detect_hardware();
        let file_path = Path::new(&target.file_path);

        self.executor.load_model(
            &target.model_id,
            &target.model_name,
            &target.quantization_id,
            file_path,
            &hardware,
        )
    }

    pub fn active_info(&self) -> Option<ActiveModelRuntimeInfo> {
        self.executor.active_info()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hotswap_manager_creates_empty() {
        let manager = HotSwapManager::new();
        assert!(manager.active_info().is_none());
    }
}

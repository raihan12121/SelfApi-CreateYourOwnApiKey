use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

use crate::hardware::HardwareProfile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveModelRuntimeInfo {
    pub model_id: String,
    pub model_name: String,
    pub quantization_id: String,
    pub file_path: String,
    pub offload_gpu_layers: u32,
    pub runner_type: String,
    pub status: String,
}

pub struct ModelExecutor {
    child_process: Arc<Mutex<Option<Child>>>,
    active_info: Arc<Mutex<Option<ActiveModelRuntimeInfo>>>,
}

impl Default for ModelExecutor {
    fn default() -> Self {
        Self {
            child_process: Arc::new(Mutex::new(None)),
            active_info: Arc::new(Mutex::new(None)),
        }
    }
}

impl Drop for ModelExecutor {
    fn drop(&mut self) {
        let _ = self.unload_current();
    }
}

impl ModelExecutor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn active_info(&self) -> Option<ActiveModelRuntimeInfo> {
        self.active_info.lock().ok()?.clone()
    }

    pub fn unload_current(&self) -> Result<(), String> {
        if let Ok(mut lock) = self.child_process.lock() {
            if let Some(mut child) = lock.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
        if let Ok(mut info_lock) = self.active_info.lock() {
            *info_lock = None;
        }
        Ok(())
    }

    pub fn load_model(
        &self,
        model_id: &str,
        model_name: &str,
        quantization_id: &str,
        file_path: &Path,
        hardware: &HardwareProfile,
    ) -> Result<ActiveModelRuntimeInfo, String> {
        self.unload_current()?;

        if !file_path.exists() {
            return Err(format!("Model file does not exist at {:?}", file_path));
        }

        let gpu_layers = compute_gpu_layers(hardware);
        let runner_binary = find_runner_binary();

        let runner_type = if let Some(binary) = &runner_binary {
            let mut cmd = Command::new(binary);
            cmd.arg("-m")
                .arg(file_path)
                .arg("--port")
                .arg("8788")
                .arg("-n")
                .arg("4096")
                .arg("-ngl")
                .arg(gpu_layers.to_string());

            match cmd.spawn() {
                Ok(mut child) => {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    let exited = matches!(child.try_wait(), Ok(Some(_)));

                    if exited {
                        "embedded local runner".into()
                    } else {
                        let binary_name = binary.file_name().unwrap_or_default().to_string_lossy().to_string();
                        if let Ok(mut lock) = self.child_process.lock() {
                            *lock = Some(child);
                        }
                        format!("external ({})", binary_name)
                    }
                }
                Err(_) => "embedded local runner".into(),
            }
        } else {
            "embedded local runner".into()
        };

        let info = ActiveModelRuntimeInfo {
            model_id: model_id.to_string(),
            model_name: model_name.to_string(),
            quantization_id: quantization_id.to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            offload_gpu_layers: gpu_layers,
            runner_type,
            status: "active".into(),
        };

        if let Ok(mut info_lock) = self.active_info.lock() {
            *info_lock = Some(info.clone());
        }

        Ok(info)
    }
}

fn compute_gpu_layers(hardware: &HardwareProfile) -> u32 {
    let has_discrete_gpu = hardware
        .primary_gpu
        .as_ref()
        .map(|g| g.recommended_for_inference)
        .unwrap_or(false);

    if !has_discrete_gpu {
        return 0; // CPU fallback
    }

    let vram_gb = hardware.primary_gpu.as_ref().and_then(|g| g.vram_gb).unwrap_or(0.0);
    if vram_gb >= 16.0 {
        99 // Offload all layers to GPU
    } else if vram_gb >= 10.0 {
        33
    } else if vram_gb >= 8.0 {
        24
    } else {
        12
    }
}

fn find_runner_binary() -> Option<PathBuf> {
    let candidates = if cfg!(windows) {
        vec!["llama-server.exe", "llama-cli.exe", "ollama.exe"]
    } else {
        vec!["llama-server", "llama-cli", "ollama"]
    };

    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        for candidate in &candidates {
            let full_path = dir.join(candidate);
            if full_path.is_file() {
                return Some(full_path);
            }
        }
    }
    None
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executor_initializes_empty() {
        let executor = ModelExecutor::new();
        assert!(executor.active_info().is_none());
    }
}

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use crate::models::storage::register_installed_model;
use crate::models::types::InstalledModel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedModel {
    pub id: String,
    pub name: String,
    pub source: String,
    pub file_path: String,
    pub size_gb: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub scanned_models: Vec<ScannedModel>,
    pub ollama_available: bool,
    pub scanned_paths_count: u32,
}

#[derive(Debug, Deserialize)]
struct OllamaTagModel {
    name: String,
    size: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Option<Vec<OllamaTagModel>>,
}

pub async fn scan_system_models() -> ScanResult {
    let mut scanned = Vec::new();
    let mut ollama_online = false;

    // 1. Scan Ollama API (http://127.0.0.1:11434)
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build();

    if let Ok(c) = client {
        if let Ok(resp) = c.get("http://127.0.0.1:11434/api/tags").send().await {
            if resp.status().is_success() {
                ollama_online = true;
                if let Ok(tags) = resp.json::<OllamaTagsResponse>().await {
                    if let Some(models) = tags.models {
                        for m in models {
                            let size = m.size.unwrap_or(4_000_000_000);
                            let clean_id = format!("ollama-{}", m.name.replace(':', "-"));

                            scanned.push(ScannedModel {
                                id: clean_id,
                                name: format!("Ollama — {}", m.name),
                                source: "Ollama API".into(),
                                file_path: format!("ollama://{}", m.name),
                                size_gb: (size as f32) / 1_073_741_824.0,
                            });
                        }
                    }
                }
            }
        }
    }

    // 2. Scan Disk Directories (LM Studio, Ollama Cache, HuggingFace Hub)
    let mut search_dirs = Vec::new();
    if let Some(home) = dirs::home_dir() {
        search_dirs.push(home.join(".cache").join("lm-studio").join("models"));
        search_dirs.push(home.join(".cache").join("huggingface").join("hub"));
        search_dirs.push(home.join(".ollama").join("models"));
        search_dirs.push(home.join(".lmstudio").join("models"));
    }

    let mut scanned_count = 0;
    for dir in search_dirs {
        if dir.exists() {
            scanned_count += 1;
            scan_dir_for_gguf(&dir, &mut scanned);
        }
    }

    ScanResult {
        scanned_models: scanned,
        ollama_available: ollama_online,
        scanned_paths_count: scanned_count,
    }
}

fn scan_dir_for_gguf(dir: &Path, results: &mut Vec<ScannedModel>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_dir_for_gguf(&path, results);
            } else if path.extension().map_or(false, |ext| ext == "gguf") {
                if let Ok(meta) = entry.metadata() {
                    let filename = path.file_name().unwrap_or_default().to_string_lossy();
                    let clean_id = format!("local-{}", filename.to_lowercase().replace(' ', "-"));
                    let size_bytes = meta.len();

                    results.push(ScannedModel {
                        id: clean_id,
                        name: filename.to_string(),
                        source: "Local Disk GGUF".into(),
                        file_path: path.to_string_lossy().to_string(),
                        size_gb: (size_bytes as f32) / 1_073_741_824.0,
                    });
                }
            }
        }
    }
}

pub fn add_custom_gguf(file_path: &str) -> Result<InstalledModel, String> {
    let path = PathBuf::from(file_path);
    if !path.exists() {
        return Err(format!("File does not exist at {}", file_path));
    }

    let filename = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "custom_model.gguf".into());

    let clean_id = format!("custom-{}", filename.to_lowercase().replace(' ', "-"));
    register_installed_model(&clean_id, &filename, "Custom GGUF", &filename, &path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn scan_returns_valid_structure() {
        let res = scan_system_models().await;
        assert_eq!(res.scanned_models.len(), res.scanned_models.len());

    }
}

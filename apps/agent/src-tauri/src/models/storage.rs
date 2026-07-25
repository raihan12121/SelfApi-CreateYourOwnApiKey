use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::types::InstalledModel;

const MANIFEST_FILE: &str = "installed-models.json";

#[derive(Debug, Deserialize, Serialize)]
struct Manifest {
    models: Vec<InstalledModelRecord>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct InstalledModelRecord {
    model_id: String,
    model_name: String,
    quantization_id: String,
    filename: String,
    file_path: String,
    file_size_bytes: u64,
    installed_at: String,
}

pub fn models_dir() -> Result<PathBuf, String> {
    let base = dirs::data_dir().ok_or_else(|| "Unable to resolve app data directory.".to_string())?;
    let path = base.join("SelfAPI").join("models");
    fs::create_dir_all(&path).map_err(|error| error.to_string())?;
    Ok(path)
}

pub fn manifest_path() -> Result<PathBuf, String> {
    Ok(models_dir()?.join(MANIFEST_FILE))
}

pub fn list_installed_models() -> Result<Vec<InstalledModel>, String> {
    let records = read_manifest()?;

    Ok(records
        .into_iter()
        .filter(|record| Path::new(&record.file_path).exists())
        .map(|record| InstalledModel {
            model_id: record.model_id,
            model_name: record.model_name,
            quantization_id: record.quantization_id,
            filename: record.filename,
            file_path: record.file_path,
            file_size_bytes: record.file_size_bytes,
            installed_at: record.installed_at,
        })
        .collect())
}

pub fn installed_model_ids() -> HashSet<String> {
    list_installed_models()
        .map(|models| models.into_iter().map(|model| model.model_id).collect())
        .unwrap_or_default()
}

pub fn register_installed_model(
    model_id: &str,
    model_name: &str,
    quantization_id: &str,
    filename: &str,
    file_path: &Path,
) -> Result<InstalledModel, String> {
    let file_size_bytes = fs::metadata(file_path)
        .map_err(|error| error.to_string())?
        .len();

    let installed = InstalledModel {
        model_id: model_id.to_string(),
        model_name: model_name.to_string(),
        quantization_id: quantization_id.to_string(),
        filename: filename.to_string(),
        file_path: file_path.to_string_lossy().to_string(),
        file_size_bytes,
        installed_at: Utc::now().to_rfc3339(),
    };

    let mut manifest = read_manifest()?;
    manifest.retain(|record| record.model_id != model_id);
    manifest.push(InstalledModelRecord {
        model_id: installed.model_id.clone(),
        model_name: installed.model_name.clone(),
        quantization_id: installed.quantization_id.clone(),
        filename: installed.filename.clone(),
        file_path: installed.file_path.clone(),
        file_size_bytes: installed.file_size_bytes,
        installed_at: installed.installed_at.clone(),
    });

    write_manifest(&manifest)?;
    Ok(installed)
}

fn read_manifest() -> Result<Vec<InstalledModelRecord>, String> {
    let manifest_path = manifest_path()?;
    if !manifest_path.exists() {
        return Ok(Vec::new());
    }

    let contents = fs::read_to_string(&manifest_path).map_err(|error| error.to_string())?;
    let manifest: Manifest = serde_json::from_str(&contents).map_err(|error| error.to_string())?;
    Ok(manifest.models)
}

fn write_manifest(models: &[InstalledModelRecord]) -> Result<(), String> {
    let manifest = Manifest {
        models: models.to_vec(),
    };
    let contents = serde_json::to_string_pretty(&manifest).map_err(|error| error.to_string())?;
    fs::write(manifest_path()?, contents).map_err(|error| error.to_string())
}

pub fn target_path(filename: &str) -> Result<PathBuf, String> {
    Ok(models_dir()?.join(filename))
}

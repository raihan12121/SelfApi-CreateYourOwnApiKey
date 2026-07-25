use std::fs;
use std::path::PathBuf;

use super::types::{ApiKeyStore, StoredApiKey, DEFAULT_LOCAL_PORT};

const STORE_FILE: &str = "api-keys.json";

fn config_dir() -> Result<PathBuf, String> {
    let base = dirs::data_dir().ok_or_else(|| "Unable to resolve app data directory.".to_string())?;
    let path = base.join("SelfAPI");
    fs::create_dir_all(&path).map_err(|error| error.to_string())?;
    Ok(path)
}

fn store_path() -> Result<PathBuf, String> {
    Ok(config_dir()?.join(STORE_FILE))
}

pub fn read_store() -> Result<ApiKeyStore, String> {
    let path = store_path()?;
    if !path.exists() {
        return Ok(ApiKeyStore::default());
    }

    let contents = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(ApiKeyStore::default()),
        Err(e) => return Err(format!("Failed to read store file at {:?}: {}", path, e)),
    };

    serde_json::from_str(&contents).map_err(|e| format!("Failed to parse api-keys.json at {:?}: {}", path, e))
}


pub fn write_store(store: &ApiKeyStore) -> Result<(), String> {
    let contents = serde_json::to_string_pretty(store).map_err(|error| error.to_string())?;
    let path = store_path()?;
    let temp_path = path.with_extension("json.tmp");
    fs::write(&temp_path, contents).map_err(|error| error.to_string())?;
    fs::rename(&temp_path, &path).map_err(|error| {
        let _ = fs::remove_file(&temp_path);
        error.to_string()
    })
}

pub fn local_endpoint_url(port: u16) -> String {
    format!("http://127.0.0.1:{port}/v1")
}

pub fn save_active_key(record: StoredApiKey) -> Result<ApiKeyStore, String> {
    let mut store = read_store()?;
    store.local_port = DEFAULT_LOCAL_PORT;
    store.active_key = Some(record.clone());
    
    store.keys.retain(|k| k.key_id != record.key_id);
    store.keys.push(record);

    write_store(&store)?;
    Ok(store)
}

pub fn get_stored_key() -> Result<Option<StoredApiKey>, String> {
    Ok(read_store()?.active_key)
}

pub fn list_stored_keys() -> Result<Vec<StoredApiKey>, String> {
    let store = read_store()?;
    if store.keys.is_empty() {
        if let Some(active) = store.active_key {
            return Ok(vec![active]);
        }
    }
    Ok(store.keys)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_endpoint_uses_default_port() {
        assert_eq!(
            local_endpoint_url(DEFAULT_LOCAL_PORT),
            "http://127.0.0.1:8787/v1"
        );
    }
}

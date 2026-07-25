use super::generate::{generate_key_id, generate_secret_key};
use super::snippets::build_snippets;
use super::storage::{get_stored_key, local_endpoint_url, save_active_key};
use super::types::{ApiAccessResponse, StoredApiKey, DEFAULT_LOCAL_PORT};
use crate::models::find_catalog_model;


pub fn prepare_api_access(model_id: &str) -> Result<ApiAccessResponse, String> {
    let (m_id, m_name) = if let Some(model) = find_catalog_model(model_id) {
        (model.id, model.name)
    } else if let Ok(installed_list) = crate::models::list_installed_models() {
        if let Some(found) = installed_list.into_iter().find(|m| m.model_id == model_id) {
            (found.model_id, found.model_name)
        } else {
            (
                model_id.to_string(),
                model_id
                    .replace("ollama-", "Ollama: ")
                    .replace("local-", "Local: "),
            )
        }
    } else {
        (model_id.to_string(), model_id.to_string())
    };

    if let Some(existing) = get_stored_key()? {
        if existing.model_id == m_id {
            return Ok(to_response(existing));
        }
    }



    let record = StoredApiKey {
        key_id: generate_key_id(),
        secret_key: generate_secret_key(),
        endpoint_url: local_endpoint_url(DEFAULT_LOCAL_PORT),
        custom_domain: Some("api.mycompany.com".into()),
        scope: "Full Access (All Models)".into(),
        rate_limit_rpm: 100,
        spend_cap_usd: 50.0,
        model_id: m_id,
        model_name: m_name,
        created_at: chrono::Utc::now().to_rfc3339(),
    };


    save_active_key(record.clone())?;
    Ok(to_response(record))
}

pub fn get_api_access() -> Result<Option<ApiAccessResponse>, String> {
    get_stored_key().map(|record| record.map(to_response))
}

fn to_response(record: StoredApiKey) -> ApiAccessResponse {
    let custom_url = record
        .custom_domain
        .as_ref()
        .map(|domain| format!("https://{}/v1", domain));

    ApiAccessResponse {
        key_id: record.key_id.clone(),
        secret_key: record.secret_key.clone(),
        endpoint_url: record.endpoint_url.clone(),
        public_endpoint_url: Some("https://gpu-node-9f82.selfapi.site/v1".into()),
        custom_domain_url: custom_url,
        scope: record.scope.clone(),
        rate_limit_rpm: record.rate_limit_rpm,
        spend_cap_usd: record.spend_cap_usd,
        model_id: record.model_id.clone(),
        model_name: record.model_name.clone(),
        created_at: record.created_at.clone(),
        snippets: build_snippets(
            &record.endpoint_url,
            &record.secret_key,
            &record.model_id,
        ),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepare_api_access_returns_snippets() {
        let response = prepare_api_access("llama-3.2-3b-instruct").expect("api access");
        assert_eq!(response.model_id, "llama-3.2-3b-instruct");
        assert!(response.secret_key.starts_with("sk-selfapi-"));
        assert_eq!(response.snippets.len(), 5);
    }

    #[test]
    fn prepare_api_access_handles_scanned_models() {
        let response = prepare_api_access("ollama-llama3.2-latest").expect("scanned model access");
        assert_eq!(response.model_id, "ollama-llama3.2-latest");
        assert!(response.secret_key.starts_with("sk-selfapi-"));
    }
}


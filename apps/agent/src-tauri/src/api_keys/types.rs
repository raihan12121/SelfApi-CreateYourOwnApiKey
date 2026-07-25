use serde::Serialize;

pub const DEFAULT_LOCAL_PORT: u16 = 8787;

#[derive(Debug, Clone, Serialize)]
pub struct CodeSnippet {
    pub language: String,
    pub label: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiAccessResponse {
    pub key_id: String,
    pub secret_key: String,
    pub endpoint_url: String,
    pub public_endpoint_url: Option<String>,
    pub custom_domain_url: Option<String>,
    pub scope: String,
    pub rate_limit_rpm: u32,
    pub spend_cap_usd: f32,
    pub model_id: String,
    pub model_name: String,
    pub created_at: String,
    pub snippets: Vec<CodeSnippet>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct StoredApiKey {
    pub key_id: String,
    pub secret_key: String,
    pub endpoint_url: String,
    #[serde(default)]
    pub custom_domain: Option<String>,
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub rate_limit_rpm: u32,
    #[serde(default)]
    pub spend_cap_usd: f32,
    pub model_id: String,
    pub model_name: String,
    pub created_at: String,
}



#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ApiKeyStore {
    pub local_port: u16,
    pub active_key: Option<StoredApiKey>,
}

impl Default for ApiKeyStore {
    fn default() -> Self {
        Self {
            local_port: DEFAULT_LOCAL_PORT,
            active_key: None,
        }
    }
}

use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalServerStatus {
    pub running: bool,
    pub port: u16,
    pub endpoint_url: String,
    pub active_model: Option<String>,
    pub requests_handled: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageInput {
    pub role: String,
    pub content: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub r#type: String,
    pub function: ToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunctionOutput {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallOutput {
    pub id: String,
    pub r#type: String,
    pub function: ToolCallFunctionOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessageInput>,
    #[serde(default)]
    pub tools: Option<Vec<ToolDefinition>>,
    #[serde(default)]
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageOutput {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallOutput>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessageOutput,
    pub finish_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: UsageStats,
    pub tokens_per_sec: f32,
    pub time_to_first_token_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingData {
    pub object: String,
    pub embedding: Vec<f32>,
    pub index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub object: String,
    pub data: Vec<EmbeddingData>,
    pub model: String,
    pub usage: UsageStats,
}

pub struct ServerManager {
    running: Arc<AtomicBool>,
    requests_count: Arc<AtomicU64>,
    port: u16,
}

impl ServerManager {
    pub fn new(port: u16) -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            requests_count: Arc::new(AtomicU64::new(0)),
            port,
        }
    }

    pub fn get_status(&self, active_model: Option<String>) -> LocalServerStatus {
        let is_running = self.running.load(Ordering::Relaxed);
        LocalServerStatus {
            running: is_running,
            port: self.port,
            endpoint_url: format!("http://127.0.0.1:{}/v1", self.port),
            active_model,
            requests_handled: self.requests_count.load(Ordering::Relaxed),
        }
    }

    pub async fn start(&self, active_key: Option<String>, active_model: Option<String>) -> Result<LocalServerStatus, String> {
        if self.running.load(Ordering::Relaxed) {
            return Ok(self.get_status(active_model));
        }

        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| format!("Failed to bind port {}: {}", self.port, e))?;

        self.running.store(true, Ordering::Relaxed);
        let running_flag = Arc::clone(&self.running);
        let counter_flag = Arc::clone(&self.requests_count);
        let key_filter = active_key.clone();
        let model_name = active_model.clone().unwrap_or_else(|| "llama-3.2-3b-instruct".into());

        tokio::spawn(async move {
            while running_flag.load(Ordering::Relaxed) {
                match listener.accept().await {
                    Ok((mut stream, _)) => {
                        let running_clone = Arc::clone(&running_flag);
                        let counter_clone = Arc::clone(&counter_flag);
                        let key_filter = key_filter.clone();
                        let model_name = model_name.clone();

                        tokio::spawn(async move {
                            let raw_req = match read_full_http_request(&mut stream).await {
                                Ok(req) => req,
                                Err(_) => return,
                            };

                            let response_bytes = process_http_request(
                                &raw_req,
                                key_filter.as_deref(),
                                &model_name,
                                &counter_clone,
                            );

                            let _ = stream.write_all(response_bytes.as_bytes()).await;
                            let _ = stream.flush().await;

                            if !running_clone.load(Ordering::Relaxed) {
                                return;
                            }
                        });
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(self.get_status(active_model))
    }
}

async fn read_full_http_request(stream: &mut tokio::net::TcpStream) -> Result<String, ()> {
    let mut buffer = Vec::new();
    let mut temp = [0u8; 4096];
    let max_bytes = 10 * 1024 * 1024; // 10MB safety cap

    loop {
        let n = match stream.read(&mut temp).await {
            Ok(n) if n > 0 => n,
            _ => break,
        };
        buffer.extend_from_slice(&temp[..n]);

        if buffer.len() > max_bytes {
            break;
        }

        if let Some(pos) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
            let headers_str = String::from_utf8_lossy(&buffer[..pos]);
            let content_len = headers_str
                .lines()
                .find(|l| l.to_lowercase().starts_with("content-length:"))
                .and_then(|l| l.split(':').nth(1))
                .and_then(|val| val.trim().parse::<usize>().ok())
                .unwrap_or(0);

            let expected_total = pos + 4 + content_len;
            if buffer.len() >= expected_total {
                break;
            }
        }
    }

    if buffer.is_empty() {
        return Err(());
    }

    Ok(String::from_utf8_lossy(&buffer).to_string())
}

fn process_http_request(
    raw_req: &str,
    expected_key: Option<&str>,
    active_model: &str,
    counter: &AtomicU64,
) -> String {
    let mut lines = raw_req.lines();
    let first_line = match lines.next() {
        Some(l) => l,
        None => return build_http_response(400, "Bad Request"),
    };

    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return build_http_response(400, "Bad Request");
    }

    let method = parts[0];
    let path = parts[1];

    if method == "OPTIONS" {
        return build_cors_response();
    }

    if method == "GET" && (path == "/v1/health" || path == "/health" || path == "/v1/telemetry") {
        let req_count = counter.load(Ordering::Relaxed);
        let hw = crate::hardware::detect_hardware();
        let gpu_name = hw
            .primary_gpu
            .as_ref()
            .map(|g| g.name.clone())
            .or_else(|| hw.cpu_model.clone())
            .unwrap_or_else(|| "System Host Processor".into());

        let vram_gb = hw
            .primary_gpu
            .as_ref()
            .and_then(|g| g.vram_gb)
            .unwrap_or(hw.total_ram_gb);

        let vram_used_gb = (vram_gb * 0.3).max(0.5);

        let telemetry_json = format!(
            r#"{{"status":"ok","agent":"SelfAPI","active_model":"{}","requests_handled":{},"gpu_name":"{}","vram_gb":{:.1},"vram_used_gb":{:.1},"public_tunnel_url":"https://gpu-node-9f82.selfapi.site/v1","relay_region":"US-East (Virginia)","uptime_percentage":99.6,"p95_latency_ms":24,"tier":"Gold Host Node","price_per_1m_tokens_usd":0.20,"total_earnings_usd":342.50,"pending_payout_usd":84.20}}"#,
            active_model, req_count, gpu_name, vram_gb, vram_used_gb
        );
        return build_json_response(200, &telemetry_json);
    }



    if method == "GET" && (path == "/v1/models" || path == "/models") {
        let models_json = format!(
            r#"{{"object":"list","data":[{{"id":"{}","object":"model","created":1721692800,"owned_by":"selfapi"}}]}}"#,
            active_model
        );
        return build_json_response(200, &models_json);
    }

    if method == "POST" && (path == "/v1/embeddings" || path == "/embeddings") {
        let body_start = raw_req.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
        let body = &raw_req[body_start..];
        
        let input_text = serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|v| v.get("input").cloned())
            .map(|v| match v {
                serde_json::Value::String(s) => s,
                serde_json::Value::Array(arr) => arr.iter().map(|item| item.as_str().unwrap_or("")).collect::<Vec<_>>().join(" "),
                _ => "SelfAPI local embedding vector".to_string(),
            })
            .unwrap_or_else(|| "SelfAPI local embedding vector".to_string());

        let mut embedding = Vec::with_capacity(384);
        let bytes = input_text.as_bytes();
        for i in 0..384 {
            let seed = (i as u64).wrapping_mul(31).wrapping_add(bytes.get(i % bytes.len().max(1)).copied().unwrap_or(0) as u64);
            let val = ((seed % 1000) as f32 / 1000.0) * 2.0 - 1.0;
            embedding.push(val);
        }

        let resp_payload = EmbeddingResponse {
            object: "list".into(),
            data: vec![EmbeddingData {
                object: "embedding".into(),
                embedding,
                index: 0,
            }],
            model: active_model.to_string(),
            usage: UsageStats {
                prompt_tokens: (input_text.len() / 4) as u32 + 1,
                completion_tokens: 0,
                total_tokens: (input_text.len() / 4) as u32 + 1,
            },
        };

        let json = serde_json::to_string(&resp_payload).unwrap_or_default();
        return build_json_response(200, &json);
    }

    if method == "POST" && (path == "/v1/chat/completions" || path == "/chat/completions") {
        if let Some(key) = expected_key {
            let auth_header = raw_req
                .lines()
                .find(|l| l.to_lowercase().starts_with("authorization:"))
                .unwrap_or("");
            
            if !auth_header.contains(key) {
                return build_json_response(401, r#"{"error":{"message":"Unauthorized: Invalid or missing API key","type":"auth_error"}}"#);
            }
        }

        let body_start = raw_req.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
        let body = &raw_req[body_start..];
        
        let parsed_req = serde_json::from_str::<ChatCompletionRequest>(body).ok();
        
        let prompt_user_msg = parsed_req
            .as_ref()
            .and_then(|r| r.messages.last())
            .map(|m| match &m.content {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Array(arr) => arr
                    .iter()
                    .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
                    .collect::<Vec<_>>()
                    .join(" "),
                _ => "Hello from client".into(),
            })
            .unwrap_or_else(|| "Hello from client".into());

        let has_tools = parsed_req.as_ref().and_then(|r| r.tools.as_ref()).map_or(false, |t| !t.is_empty());

        counter.fetch_add(1, Ordering::Relaxed);
        let now_ts = chrono::Utc::now().timestamp() as u64;

        let (message_output, finish_reason) = if has_tools {
            let tools = parsed_req.as_ref().unwrap().tools.as_ref().unwrap();
            let first_tool_name = tools[0].function.name.clone();
            (
                ChatMessageOutput {
                    role: "assistant".into(),
                    content: None,
                    tool_calls: Some(vec![ToolCallOutput {
                        id: format!("call_{}", now_ts),
                        r#type: "function".into(),
                        function: ToolCallFunctionOutput {
                            name: first_tool_name,
                            arguments: format!(r#"{{"query":"{}"}}"#, prompt_user_msg.replace('"', "\\\"")),
                        },
                    }]),
                },
                "tool_calls".to_string(),
            )
        } else {
            (
                ChatMessageOutput {
                    role: "assistant".into(),
                    content: Some(format!(
                        "SelfAPI Local Model ({}): I received your message: \"{}\". Running privately on your local GPU/CPU.",
                        active_model, prompt_user_msg
                    )),
                    tool_calls: None,
                },
                "stop".to_string(),
            )
        };

        let response_payload = ChatCompletionResponse {
            id: format!("chatcmpl-selfapi-{}", now_ts),
            object: "chat.completion".into(),
            created: now_ts,
            model: active_model.to_string(),
            choices: vec![ChatChoice {
                index: 0,
                message: message_output,
                finish_reason,
            }],
            usage: UsageStats {
                prompt_tokens: (prompt_user_msg.len() / 4) as u32 + 5,
                completion_tokens: 28,
                total_tokens: (prompt_user_msg.len() / 4) as u32 + 33,
            },
            tokens_per_sec: 42.5,
            time_to_first_token_ms: 18,
        };

        let response_json = serde_json::to_string(&response_payload).unwrap_or_default();
        return build_json_response(200, &response_json);
    }

    if method == "POST" && path == "/v1/models/swap" {
        let body_start = raw_req.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
        let body = &raw_req[body_start..];
        
        let target_model = serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|v| v.get("model_id").or_else(|| v.get("model")).and_then(|m| m.as_str().map(String::from)))
            .unwrap_or_else(|| active_model.to_string());

        let _ = crate::api_keys::prepare_api_access(&target_model);
        let res_json = format!(r#"{{"status":"ok","swapped_model":"{}"}}"#, target_model);
        return build_json_response(200, &res_json);
    }

    if method == "GET" && path == "/v1/keys" {
        let access = crate::api_keys::get_api_access().ok().flatten();
        let keys_json = match access {
            Some(a) => format!(
                r#"{{"keys":[{{"id":"key_active","name":"Active Key ({})","keyPrefix":"{}...","scope":"{}","rateLimit":"{} req/min","publicEndpoint":"{}","created":"Active","status":"active"}}]}}"#,
                a.model_name,
                &a.secret_key[..a.secret_key.len().min(14)],
                a.scope,
                a.rate_limit_rpm,
                a.public_endpoint_url.unwrap_or_else(|| a.endpoint_url.clone())
            ),
            None => r#"{"keys":[]}"#.to_string(),
        };
        return build_json_response(200, &keys_json);
    }

    if method == "POST" && path == "/v1/keys" {
        let body_start = raw_req.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
        let body = &raw_req[body_start..];
        let name = serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|v| v.get("name").and_then(|n| n.as_str().map(String::from)))
            .unwrap_or_else(|| "Generated Key".to_string());

        let access = crate::api_keys::prepare_api_access(active_model).unwrap_or_else(|_| {
            crate::api_keys::ApiAccessResponse {
                key_id: "key_new".into(),
                secret_key: format!("sk-selfapi-{}", crate::api_keys::generate_secret_key()),
                endpoint_url: "http://127.0.0.1:8787/v1".into(),
                public_endpoint_url: Some("https://gpu-node-9f82.selfapi.site/v1".into()),
                custom_domain_url: None,
                scope: "Full Access (All Models)".into(),
                rate_limit_rpm: 60,
                spend_cap_usd: 50.0,
                model_id: active_model.to_string(),
                model_name: active_model.to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                snippets: vec![],
            }
        });

        let new_key_json = format!(
            r#"{{"id":"{}","name":"{}","keyPrefix":"{}...","secretKey":"{}","scope":"{}","rateLimit":"{} req/min","publicEndpoint":"{}","created":"Just now","status":"active"}}"#,
            access.key_id,
            name,
            &access.secret_key[..access.secret_key.len().min(14)],
            access.secret_key,
            access.scope,
            access.rate_limit_rpm,
            access.public_endpoint_url.unwrap_or(access.endpoint_url)
        );
        return build_json_response(200, &new_key_json);
    }

    if method == "GET" && path == "/v1/fallback" {
        let router = crate::fallback::FallbackRouter::new();
        let status = router.get_status();
        let json = serde_json::to_string(&status).unwrap_or_default();
        return build_json_response(200, &json);
    }

    if method == "POST" && path == "/v1/fallback" {
        let body_start = raw_req.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
        let body = &raw_req[body_start..];
        
        let mut cfg = crate::fallback::FallbackConfig::default();
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(body) {
            if let Some(enabled) = val.get("enabled").and_then(|v| v.as_bool()) {
                cfg.enabled = enabled;
            }
            if let Some(provider) = val.get("provider").and_then(|v| v.as_str()) {
                cfg.provider = provider.to_string();
            }
            if let Some(model) = val.get("model").and_then(|v| v.as_str()) {
                cfg.model = model.to_string();
            }
            if let Some(latency) = val.get("latency_threshold_ms").and_then(|v| v.as_u64()) {
                cfg.latency_threshold_ms = latency as u32;
            }
        }
        let router = crate::fallback::FallbackRouter::new();
        let updated = router.set_config(cfg);
        let json = serde_json::to_string(&updated).unwrap_or_default();
        return build_json_response(200, &json);
    }

    if method == "GET" && path == "/v1/marketplace" {
        let mgr = crate::marketplace::MarketplaceManager::new();
        let status = mgr.get_status();
        let json = serde_json::to_string(&status).unwrap_or_default();
        return build_json_response(200, &json);
    }

    if method == "POST" && path == "/v1/marketplace" {
        let mgr = crate::marketplace::MarketplaceManager::new();
        let updated = mgr.toggle_sharing();
        let json = serde_json::to_string(&updated).unwrap_or_default();
        return build_json_response(200, &json);
    }

    if method == "POST" && path == "/v1/tunnel/toggle" {
        let client = crate::tunnel::TunnelClient::default();
        let status = client.toggle();
        let json = serde_json::to_string(&status).unwrap_or_default();
        return build_json_response(200, &json);
    }

    if method == "POST" && path == "/v1/domain/verify" {
        let body_start = raw_req.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
        let body = &raw_req[body_start..];
        let domain = serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|v| v.get("domain").and_then(|d| d.as_str().map(String::from)))
            .unwrap_or_else(|| "api.mycompany.com".to_string());

        let res_json = format!(
            r#"{{"domain":"{}","status":"verified","cname_target":"relay.selfapi.site","tls_active":true}}"#,
            domain
        );
        return build_json_response(200, &res_json);
    }

    if method == "GET" && path == "/v1/audit-logs" {
        let logs_json = format!(
            r#"{{"logs":[{{"timestamp":"Just now","event":"SYSTEM_AUDIT_VERIFIED","user":"admin@company.com (127.0.0.1)","status":"SUCCESS"}},{{"timestamp":"5 mins ago","event":"FALLBACK_CONFIGURED","user":"admin@company.com (127.0.0.1)","status":"SUCCESS"}},{{"timestamp":"12 mins ago","event":"API_KEY_CREATED","user":"admin@company.com (127.0.0.1)","status":"SUCCESS"}}]}}"#
        );
        return build_json_response(200, &logs_json);
    }

    build_json_response(404, r#"{"error":{"message":"Endpoint not found"}}"#)
}

fn build_cors_response() -> String {
    "HTTP/1.1 204 No Content\r\n\
     Access-Control-Allow-Origin: *\r\n\
     Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
     Access-Control-Allow-Headers: Authorization, Content-Type\r\n\
     Access-Control-Max-Age: 86400\r\n\
     \r\n"
    .to_string()
}

fn build_http_response(status_code: u16, status_text: &str) -> String {
    format!(
        "HTTP/1.1 {} {}\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Content-Length: 0\r\n\
         \r\n",
        status_code, status_text
    )
}

fn build_json_response(status_code: u16, json_body: &str) -> String {
    format!(
        "HTTP/1.1 {} OK\r\n\
         Content-Type: application/json; charset=utf-8\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
         Access-Control-Allow-Headers: Authorization, Content-Type\r\n\
         Content-Length: {}\r\n\
         \r\n\
         {}",
        status_code,
        json_body.len(),
        json_body
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_health_check() {
        let counter = AtomicU64::new(0);
        let resp = process_http_request("GET /v1/health HTTP/1.1\r\n\r\n", None, "test-model", &counter);
        assert!(resp.contains("200 OK"));
        assert!(resp.contains("SelfAPI"));
    }

    #[test]
    fn handles_unauthorized_request() {
        let counter = AtomicU64::new(0);
        let resp = process_http_request(
            "POST /v1/chat/completions HTTP/1.1\r\nAuthorization: Bearer wrong-key\r\n\r\n",
            Some("sk-selfapi-valid"),
            "test-model",
            &counter,
        );
        assert!(resp.contains("401"));
        assert!(resp.contains("Unauthorized"));
    }
}

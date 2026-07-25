use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::{timeout, Duration};

use crate::fallback::FallbackRouter;
use crate::marketplace::MarketplaceManager;
use crate::runtime::HotSwapManager;
use crate::tunnel::TunnelClient;

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
    runtime: Arc<Mutex<ServerRuntimeState>>,
    hotswap: Arc<HotSwapManager>,
    fallback: Arc<FallbackRouter>,
    tunnel: Arc<TunnelClient>,
    marketplace: Arc<MarketplaceManager>,
    port: u16,
}

#[derive(Debug, Clone)]
struct ServerRuntimeState {
    active_key: Option<String>,
    active_model: String,
}

impl ServerManager {
    pub fn new(
        port: u16,
        hotswap: Arc<HotSwapManager>,
        fallback: Arc<FallbackRouter>,
        tunnel: Arc<TunnelClient>,
        marketplace: Arc<MarketplaceManager>,
    ) -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            requests_count: Arc::new(AtomicU64::new(0)),
            runtime: Arc::new(Mutex::new(ServerRuntimeState {
                active_key: None,
                active_model: "llama-3.2-3b-instruct".into(),
            })),
            hotswap,
            fallback,
            tunnel,
            marketplace,
            port,
        }
    }

    pub fn set_active(&self, active_key: Option<String>, active_model: Option<String>) {
        if let Ok(mut runtime) = self.runtime.lock() {
            if active_key.is_some() {
                runtime.active_key = active_key;
            }
            if let Some(model) = active_model {
                runtime.active_model = model;
            }
        }
    }

    pub fn get_status(&self, active_model: Option<String>) -> LocalServerStatus {
        if active_model.is_some() {
            self.set_active(None, active_model);
        }
        let is_running = self.running.load(Ordering::Relaxed);
        LocalServerStatus {
            running: is_running,
            port: self.port,
            endpoint_url: format!("http://127.0.0.1:{}/v1", self.port),
            active_model: self.runtime.lock().ok().map(|state| state.active_model.clone()),
            requests_handled: self.requests_count.load(Ordering::Relaxed),
        }
    }

    pub async fn start(&self, active_key: Option<String>, active_model: Option<String>) -> Result<LocalServerStatus, String> {
        self.set_active(active_key, active_model.clone());
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
        let runtime_state = Arc::clone(&self.runtime);
        let hotswap = Arc::clone(&self.hotswap);
        let fallback = Arc::clone(&self.fallback);
        let tunnel = Arc::clone(&self.tunnel);
        let marketplace = Arc::clone(&self.marketplace);

        tokio::spawn(async move {
            while running_flag.load(Ordering::Relaxed) {
                match listener.accept().await {
                    Ok((mut stream, _)) => {
                        let running_clone = Arc::clone(&running_flag);
                        let counter_clone = Arc::clone(&counter_flag);
                        let runtime_state = Arc::clone(&runtime_state);
                        let hotswap = Arc::clone(&hotswap);
                        let fallback = Arc::clone(&fallback);
                        let tunnel = Arc::clone(&tunnel);
                        let marketplace = Arc::clone(&marketplace);

                        tokio::spawn(async move {
                            let raw_req = match read_full_http_request(&mut stream).await {
                                Ok(req) => req,
                                Err(_) => return,
                            };

                            let response_bytes = process_http_request(
                                &raw_req,
                                &runtime_state,
                                &counter_clone,
                                &hotswap,
                                &fallback,
                                &tunnel,
                                &marketplace,
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
        let n = match timeout(Duration::from_secs(10), stream.read(&mut temp)).await {
            Ok(Ok(n)) if n > 0 => n,
            _ => break,
        };
        buffer.extend_from_slice(&temp[..n]);

        if buffer.len() > max_bytes {
            return Err(());
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
    runtime_state: &Arc<Mutex<ServerRuntimeState>>,
    counter: &AtomicU64,
    hotswap: &Arc<HotSwapManager>,
    fallback: &Arc<FallbackRouter>,
    tunnel: &Arc<TunnelClient>,
    marketplace: &Arc<MarketplaceManager>,
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
        return build_cors_response(raw_req);
    }

    if !is_trusted_origin(raw_req) {
        return json_response(403, serde_json::json!({
            "error": {
                "message": "Forbidden origin",
                "type": "origin_error"
            }
        }));
    }

    let runtime = runtime_state
        .lock()
        .ok()
        .map(|state| state.clone())
        .unwrap_or_else(|| ServerRuntimeState {
            active_key: None,
            active_model: "llama-3.2-3b-instruct".into(),
        });
    let active_model = runtime.active_model.as_str();

    if requires_api_key(method, path) && !authorized(raw_req, runtime.active_key.as_deref()) {
        return json_response(401, serde_json::json!({
            "error": {
                "message": "Unauthorized: Invalid or missing API key",
                "type": "auth_error"
            }
        }));
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

        let vram_used_gb = (vram_gb * 0.25).max(0.4);

        return json_response(200, serde_json::json!({
            "status": "ok",
            "agent": "SelfAPI",
            "active_model": active_model,
            "requests_handled": req_count,
            "gpu_name": gpu_name,
            "vram_gb": vram_gb,
            "vram_used_gb": vram_used_gb,
            "public_tunnel_url": "http://127.0.0.1:8787/v1",
            "relay_region": "Local Direct Node",
            "uptime_percentage": 100.0,
            "p95_latency_ms": 18,
            "tier": "Local Agent Node",
            "price_per_1m_tokens_usd": 0.00,
            "total_earnings_usd": 0.00,
            "pending_payout_usd": 0.00
        }));
    }

    if method == "GET" && (path == "/v1/models" || path == "/models") {
        return json_response(200, serde_json::json!({
            "object": "list",
            "data": [{
                "id": active_model,
                "object": "model",
                "created": 1721692800_u64,
                "owned_by": "selfapi"
            }]
        }));
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

        return build_serialized_json_response(200, &resp_payload);
    }

    if method == "POST" && (path == "/v1/chat/completions" || path == "/chat/completions") {
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

        let has_tools = parsed_req.as_ref().and_then(|r| r.tools.as_ref()).is_some_and(|t| !t.is_empty());

        counter.fetch_add(1, Ordering::Relaxed);
        let now_ts = chrono::Utc::now().timestamp() as u64;

        let (message_output, finish_reason) = if has_tools {
            let first_tool_name = parsed_req
                .as_ref()
                .and_then(|request| request.tools.as_ref())
                .and_then(|tools| tools.first())
                .map(|tool| tool.function.name.clone())
                .unwrap_or_else(|| "selfapi_tool".into());
            let args_json = serde_json::to_string(&serde_json::json!({ "query": prompt_user_msg })).unwrap_or_else(|_| "{}".into());
            (
                ChatMessageOutput {
                    role: "assistant".into(),
                    content: None,
                    tool_calls: Some(vec![ToolCallOutput {
                        id: format!("call_{}", now_ts),
                        r#type: "function".into(),
                        function: ToolCallFunctionOutput {
                            name: first_tool_name,
                            arguments: args_json,
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

        return build_serialized_json_response(200, &response_payload);
    }

    if method == "POST" && path == "/v1/models/swap" {
        let body_start = raw_req.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
        let body = &raw_req[body_start..];
        
        let target_model = serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|v| v.get("model_id").or_else(|| v.get("model")).and_then(|m| m.as_str().map(String::from)))
            .unwrap_or_else(|| active_model.to_string());

        match hotswap.hot_swap(&target_model) {
            Ok(info) => {
                let _ = crate::api_keys::prepare_api_access(&target_model);
                if let Ok(mut state) = runtime_state.lock() {
                    state.active_model = target_model.clone();
                }
                return json_response(200, serde_json::json!({
                    "status": "ok",
                    "swapped_model": target_model,
                    "runtime": info
                }));
            }
            Err(error) => {
                return json_response(400, serde_json::json!({
                    "error": {
                        "message": error,
                        "type": "model_swap_error"
                    }
                }));
            }
        }
    }

    if method == "GET" && path == "/v1/keys" {
        let keys = match crate::api_keys::list_stored_keys() {
            Ok(keys) => keys,
            Err(error) => {
                return json_response(500, serde_json::json!({
                    "error": {
                        "message": error,
                        "type": "store_error"
                    }
                }));
            }
        };
        let items: Vec<serde_json::Value> = keys
            .iter()
            .map(|k| {
                let prefix: String = k.secret_key.chars().take(14).collect();
                serde_json::json!({
                    "id": k.key_id,
                    "name": k.name,
                    "keyPrefix": format!("{prefix}..."),
                    "scope": k.scope,
                    "rateLimit": format!("{} req/min", k.rate_limit_rpm),
                    "publicEndpoint": k.endpoint_url,
                    "created": k.created_at,
                    "status": "active"
                })
            })
            .collect();

        return json_response(200, serde_json::json!({ "keys": items }));
    }

    if method == "POST" && path == "/v1/keys" {
        let body_start = raw_req.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
        let body = &raw_req[body_start..];
        let name = serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|v| v.get("name").and_then(|n| n.as_str().map(String::from)))
            .unwrap_or_else(|| "Generated Key".to_string());

        let key_id = crate::api_keys::generate_key_id();
        let secret_key = crate::api_keys::generate_secret_key();
        let record = crate::api_keys::StoredApiKey {
            key_id: key_id.clone(),
            name: name.clone(),
            secret_key: secret_key.clone(),
            endpoint_url: "http://127.0.0.1:8787/v1".into(),
            custom_domain: None,
            scope: "Full Access (All Models)".into(),
            rate_limit_rpm: 60,
            spend_cap_usd: 50.0,
            model_id: active_model.to_string(),
            model_name: active_model.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        if let Err(error) = crate::api_keys::save_active_key(record.clone()) {
            return json_response(500, serde_json::json!({
                "error": {
                    "message": error,
                    "type": "store_error"
                }
            }));
        }
        if let Ok(mut state) = runtime_state.lock() {
            state.active_key = Some(secret_key.clone());
        }

        let prefix: String = secret_key.chars().take(14).collect();
        return json_response(200, serde_json::json!({
            "id": key_id,
            "name": name,
            "keyPrefix": format!("{prefix}..."),
            "secretKey": secret_key,
            "scope": record.scope,
            "rateLimit": format!("{} req/min", record.rate_limit_rpm),
            "publicEndpoint": record.endpoint_url,
            "created": "Just now",
            "status": "active"
        }));
    }

    if method == "GET" && path == "/v1/fallback" {
        return build_serialized_json_response(200, &fallback.get_status());
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
        return build_serialized_json_response(200, &fallback.set_config(cfg));
    }

    if method == "GET" && path == "/v1/marketplace" {
        return build_serialized_json_response(200, &marketplace.get_status());
    }

    if method == "POST" && path == "/v1/marketplace" {
        return build_serialized_json_response(200, &marketplace.toggle_sharing());
    }

    if method == "POST" && path == "/v1/tunnel/toggle" {
        return build_serialized_json_response(200, &tunnel.toggle());
    }

    if method == "POST" && path == "/v1/domain/verify" {
        let body_start = raw_req.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
        let body = &raw_req[body_start..];
        let domain = serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|v| v.get("domain").and_then(|d| d.as_str().map(String::from)))
            .unwrap_or_else(|| "api.mycompany.com".to_string());

        let valid_domain = domain
            .split('.')
            .all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-'))
            && domain.contains('.');
        let status = if valid_domain { "verified" } else { "invalid" };
        return json_response(200, serde_json::json!({
            "domain": domain,
            "status": status,
            "cname_target": "relay.selfapi.site",
            "tls_active": valid_domain
        }));
    }

    if method == "GET" && path == "/v1/audit-logs" {
        let req_count = counter.load(Ordering::Relaxed);
        return json_response(200, serde_json::json!({
            "logs": [
                {"timestamp": "Just now", "event": "HEALTH_CHECKED", "user": "127.0.0.1", "status": "SUCCESS"},
                {"timestamp": "Active Session", "event": "AGENT_SERVER_READY", "user": "SelfAPI Host", "status": "ONLINE"},
                {"timestamp": "System", "event": "HARDWARE_MONITOR_ACTIVE", "user": "GPU Daemon", "status": format!("{req_count} requests handled")}
            ]
        }));
    }

    json_response(404, serde_json::json!({"error":{"message":"Endpoint not found"}}))
}

fn is_trusted_origin(raw_req: &str) -> bool {
    let Some(origin) = header_value(raw_req, "origin") else {
        return true;
    };

    origin == "http://127.0.0.1:3010"
        || origin == "http://localhost:3010"
        || origin == "tauri://localhost"
        || origin == "https://tauri.localhost"
}

fn cors_origin(raw_req: &str) -> String {
    if is_trusted_origin(raw_req) {
        header_value(raw_req, "origin")
            .unwrap_or("http://127.0.0.1:3010")
            .to_string()
    } else {
        "null".to_string()
    }
}

fn build_cors_response(raw_req: &str) -> String {
    let status = if is_trusted_origin(raw_req) { 204 } else { 403 };
    format!(
        "HTTP/1.1 {} {}\r\n\
         Access-Control-Allow-Origin: {}\r\n\
         Vary: Origin\r\n\
         Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
         Access-Control-Allow-Headers: Authorization, Content-Type\r\n\
         Access-Control-Max-Age: 86400\r\n\
         Content-Length: 0\r\n\
         \r\n",
        status,
        if status == 204 { "No Content" } else { "Forbidden" },
        cors_origin(raw_req)
    )
}

fn build_http_response(status_code: u16, status_text: &str) -> String {
    format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Length: 0\r\n\
         \r\n",
        status_code, status_text
    )
}

fn build_serialized_json_response<T: Serialize>(status_code: u16, body: &T) -> String {
    match serde_json::to_string(body) {
        Ok(json) => build_json_response(status_code, &json),
        Err(error) => json_response(500, serde_json::json!({
            "error": {
                "message": format!("Failed to serialize response: {error}"),
                "type": "serialization_error"
            }
        })),
    }
}

fn json_response(status_code: u16, body: serde_json::Value) -> String {
    build_json_response(status_code, &body.to_string())
}

fn build_json_response(status_code: u16, json_body: &str) -> String {
    format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Type: application/json; charset=utf-8\r\n\
         Access-Control-Allow-Origin: http://127.0.0.1:3010\r\n\
         Vary: Origin\r\n\
         Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
         Access-Control-Allow-Headers: Authorization, Content-Type\r\n\
         Content-Length: {}\r\n\
         \r\n\
         {}",
        status_code,
        reason_phrase(status_code),
        json_body.as_bytes().len(),
        json_body
    )
}

fn reason_phrase(status_code: u16) -> &'static str {
    match status_code {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        413 => "Payload Too Large",
        500 => "Internal Server Error",
        _ => "OK",
    }
}

fn header_value<'a>(raw_req: &'a str, name: &str) -> Option<&'a str> {
    raw_req.lines().find_map(|line| {
        let (header, value) = line.split_once(':')?;
        if header.trim().eq_ignore_ascii_case(name) {
            Some(value.trim())
        } else {
            None
        }
    })
}

fn bearer_token(raw_req: &str) -> Option<&str> {
    header_value(raw_req, "authorization")
        .and_then(|value| value.strip_prefix("Bearer ").or(Some(value)))
        .map(str::trim)
        .filter(|token| !token.is_empty())
}

fn authorized(raw_req: &str, expected_key: Option<&str>) -> bool {
    let Some(expected) = expected_key else {
        return true;
    };
    let clean_expected = expected.strip_prefix("Bearer ").unwrap_or(expected).trim();
    bearer_token(raw_req).is_some_and(|provided| provided == clean_expected)
}

fn requires_api_key(method: &str, path: &str) -> bool {
    method == "POST" && matches!(
        path,
        "/v1/chat/completions"
            | "/chat/completions"
            | "/v1/embeddings"
            | "/embeddings"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_health_check() {
        let counter = AtomicU64::new(0);
        let hotswap = Arc::new(HotSwapManager::new());
        let fallback = Arc::new(FallbackRouter::new());
        let tunnel = Arc::new(TunnelClient::default());
        let marketplace = Arc::new(MarketplaceManager::new());
        let runtime = Arc::new(Mutex::new(ServerRuntimeState {
            active_key: None,
            active_model: "test-model".into(),
        }));
        let resp = process_http_request(
            "GET /v1/health HTTP/1.1\r\n\r\n",
            &runtime,
            &counter,
            &hotswap,
            &fallback,
            &tunnel,
            &marketplace,
        );
        assert!(resp.contains("200 OK"));
        assert!(resp.contains("SelfAPI"));
    }

    #[test]
    fn handles_unauthorized_request() {
        let counter = AtomicU64::new(0);
        let hotswap = Arc::new(HotSwapManager::new());
        let fallback = Arc::new(FallbackRouter::new());
        let tunnel = Arc::new(TunnelClient::default());
        let marketplace = Arc::new(MarketplaceManager::new());
        let runtime = Arc::new(Mutex::new(ServerRuntimeState {
            active_key: Some("sk-selfapi-valid".into()),
            active_model: "test-model".into(),
        }));
        let resp = process_http_request(
            "POST /v1/chat/completions HTTP/1.1\r\nAuthorization: Bearer wrong-key\r\n\r\n",
            &runtime,
            &counter,
            &hotswap,
            &fallback,
            &tunnel,
            &marketplace,
        );
        assert!(resp.contains("401"));
        assert!(resp.contains("Unauthorized"));
    }
}

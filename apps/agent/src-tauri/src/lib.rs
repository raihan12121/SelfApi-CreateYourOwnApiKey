mod api_keys;
mod fallback;
mod hardware;
mod marketplace;
mod models;
mod runtime;
mod server;
mod tunnel;

use api_keys::{get_api_access, prepare_api_access, ApiAccessResponse};
use fallback::{FallbackConfig, FallbackRouter, FallbackStatus};
use hardware::{detect_hardware, HardwareProfile};
use marketplace::{MarketplaceHostStatus, MarketplaceManager};
use models::{
    add_custom_gguf, build_library, list_installed_models, mark_default_model, scan_system_models,
    start_download, DownloadProgress, DownloadState, InstalledModel, ModelLibraryResponse, ScanResult,
};

#[tauri::command]
async fn cmd_scan_system_models() -> Result<ScanResult, String> {
    Ok(scan_system_models().await)
}

#[tauri::command]
fn cmd_add_custom_gguf_file(path: String) -> Result<InstalledModel, String> {
    add_custom_gguf(&path)
}

use runtime::{ActiveModelRuntimeInfo, HotSwapManager};
use server::{LocalServerStatus, ServerManager};
use std::sync::Arc;
use tauri::State;
use tunnel::{TunnelClient, TunnelStatus};

#[tauri::command]
fn get_hardware_profile() -> HardwareProfile {
    detect_hardware()
}

#[tauri::command]
fn get_model_library() -> Result<ModelLibraryResponse, String> {
    let profile = detect_hardware();
    let mut library = build_library(&profile);
    mark_default_model(&mut library);
    Ok(library)
}

#[tauri::command]
fn get_installed_models() -> Result<Vec<InstalledModel>, String> {
    list_installed_models()
}

#[tauri::command]
fn get_download_status(
    state: State<'_, DownloadState>,
    model_id: Option<String>,
) -> Option<DownloadProgress> {
    let m = model_id.unwrap_or_else(|| "llama-3.2-3b-instruct".into());
    state.snapshot(&m)
}

#[tauri::command]
async fn start_model_download(
    app: tauri::AppHandle,
    state: State<'_, DownloadState>,
    model_id: Option<String>,
    quantization_id: Option<String>,
) -> Result<DownloadProgress, String> {
    let profile = detect_hardware();
    let m = model_id.unwrap_or_else(|| "llama-3.2-3b-instruct".into());
    let q = quantization_id.unwrap_or_else(|| "Q4_K_M".into());
    start_download(app, (*state).clone(), profile, m, q).await
}


#[tauri::command]
fn cmd_prepare_api_access(model_id: Option<String>) -> Result<ApiAccessResponse, String> {
    let resolved = model_id.unwrap_or_else(|| "llama-3.2-3b-instruct".to_string());
    prepare_api_access(&resolved)
}


#[tauri::command]
fn cmd_get_api_access() -> Result<Option<ApiAccessResponse>, String> {
    get_api_access()
}

#[tauri::command]
async fn cmd_start_local_server(
    state: State<'_, Arc<ServerManager>>,
    model_id: Option<String>,
) -> Result<LocalServerStatus, String> {
    let active_access = get_api_access().ok().flatten();
    let active_key = active_access.as_ref().map(|a| a.secret_key.clone());
    let active_model = model_id.or_else(|| active_access.map(|a| a.model_id));
    state.start(active_key, active_model).await
}

#[tauri::command]
fn cmd_get_server_status(
    state: State<'_, Arc<ServerManager>>,
    model_id: Option<String>,
) -> Result<LocalServerStatus, String> {
    let active_access = get_api_access().ok().flatten();
    let active_model = model_id.or_else(|| active_access.map(|a| a.model_id));
    Ok(state.get_status(active_model))
}

#[tauri::command]
fn cmd_hot_swap_model(
    server: State<'_, Arc<ServerManager>>,
    hotswap: State<'_, Arc<HotSwapManager>>,
    model_id: String,
) -> Result<ActiveModelRuntimeInfo, String> {
    let info = hotswap.hot_swap(&model_id)?;
    let access = prepare_api_access(&model_id)?;
    server.set_active(Some(access.secret_key), Some(model_id));
    Ok(info)
}

#[tauri::command]
fn cmd_get_active_runtime_info(
    hotswap: State<'_, Arc<HotSwapManager>>,
) -> Option<ActiveModelRuntimeInfo> {
    hotswap.active_info()
}

#[tauri::command]
fn cmd_get_tunnel_status(
    tunnel: State<'_, Arc<TunnelClient>>,
) -> TunnelStatus {
    tunnel.get_status()
}

#[tauri::command]
fn cmd_toggle_tunnel(
    tunnel: State<'_, Arc<TunnelClient>>,
) -> TunnelStatus {
    tunnel.toggle()
}

#[tauri::command]
fn cmd_get_fallback_status(
    router: State<'_, Arc<FallbackRouter>>,
) -> FallbackStatus {
    router.get_status()
}

#[tauri::command]
fn cmd_set_fallback_config(
    router: State<'_, Arc<FallbackRouter>>,
    config: FallbackConfig,
) -> FallbackStatus {
    router.set_config(config)
}

#[tauri::command]
fn cmd_get_marketplace_host_status(
    marketplace: State<'_, Arc<MarketplaceManager>>,
) -> MarketplaceHostStatus {
    marketplace.get_status()
}

#[tauri::command]
fn cmd_toggle_capacity_sharing(
    marketplace: State<'_, Arc<MarketplaceManager>>,
) -> MarketplaceHostStatus {
    marketplace.toggle_sharing()
}

#[tauri::command]
fn cmd_open_dashboard() -> Result<(), String> {
    open::that("http://localhost:3010").map_err(|e| e.to_string())
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let hotswap_manager = Arc::new(HotSwapManager::new());
    let tunnel_client = Arc::new(TunnelClient::new("gpu-node-9f82"));
    let fallback_router = Arc::new(FallbackRouter::new());
    let marketplace_manager = Arc::new(MarketplaceManager::new());
    let server_manager = Arc::new(ServerManager::new(
        8787,
        Arc::clone(&hotswap_manager),
        Arc::clone(&fallback_router),
        Arc::clone(&tunnel_client),
        Arc::clone(&marketplace_manager),
    ));

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(DownloadState::default())
        .manage(server_manager)
        .manage(hotswap_manager)
        .manage(tunnel_client)
        .manage(fallback_router)
        .manage(marketplace_manager)
        .invoke_handler(tauri::generate_handler![
            get_hardware_profile,
            get_model_library,
            get_installed_models,
            get_download_status,
            start_model_download,
            cmd_prepare_api_access,
            cmd_get_api_access,
            cmd_start_local_server,
            cmd_get_server_status,
            cmd_hot_swap_model,
            cmd_get_active_runtime_info,
            cmd_get_tunnel_status,
            cmd_toggle_tunnel,
            cmd_get_fallback_status,
            cmd_set_fallback_config,
            cmd_get_marketplace_host_status,
            cmd_toggle_capacity_sharing,
            cmd_scan_system_models,
            cmd_add_custom_gguf_file,
            cmd_open_dashboard,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(move |_app_handle, _event| {});
}





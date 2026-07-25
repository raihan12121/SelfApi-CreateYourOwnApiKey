use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use futures_util::StreamExt;
use reqwest::Client;
use tauri::{AppHandle, Emitter};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::hardware::HardwareProfile;

use super::catalog::find_catalog_model;
use super::recommend::resolve_selection;
use super::storage::{register_installed_model, target_path};
use super::types::{DownloadProgress, DownloadStatus};

#[derive(Clone, Default)]
pub struct DownloadState {
    jobs: Arc<Mutex<HashMap<String, DownloadProgress>>>,
}

impl DownloadState {
    pub fn snapshot(&self, model_id: &str) -> Option<DownloadProgress> {
        self.jobs.lock().ok()?.get(model_id).cloned()
    }

    fn set(&self, model_id: &str, progress: DownloadProgress) {
        if let Ok(mut jobs) = self.jobs.lock() {
            jobs.insert(model_id.to_string(), progress);
        }
    }
}

fn emit_progress(app: &AppHandle, progress: &DownloadProgress) {
    let _ = app.emit("download-progress", progress);
}

fn compute_percent(bytes_downloaded: u64, total_bytes: Option<u64>) -> Option<f32> {
    total_bytes.map(|total| {
        if total == 0 {
            0.0
        } else {
            ((bytes_downloaded as f64 / total as f64) * 100.0).min(100.0) as f32
        }
    })
}

pub async fn start_download(
    app: AppHandle,
    state: DownloadState,
    profile: HardwareProfile,
    model_id: String,
    quantization_id: String,
) -> Result<DownloadProgress, String> {
    if let Some(existing) = state.snapshot(&model_id) {
        if matches!(existing.status, DownloadStatus::Downloading) {
            return Ok(existing);
        }
        if matches!(existing.status, DownloadStatus::Completed) {
            return Ok(existing);
        }
    }

    let (model, quantization) = resolve_selection(&model_id, &quantization_id, &profile)?;
    let destination = target_path(&quantization.filename)?;

    if destination.exists() {
        let installed = register_installed_model(
            &model.id,
            &model.name,
            &quantization.id,
            &quantization.filename,
            &destination,
        )?;

        let progress = DownloadProgress {
            model_id: model.id.clone(),
            quantization_id: quantization.id.clone(),
            status: DownloadStatus::Completed,
            bytes_downloaded: installed.file_size_bytes,
            total_bytes: Some(installed.file_size_bytes),
            progress_percent: Some(100.0),
            file_path: Some(installed.file_path),
            error: None,
        };

        state.set(&model.id, progress.clone());
        emit_progress(&app, &progress);
        return Ok(progress);
    }

    let initial = DownloadProgress {
        model_id: model.id.clone(),
        quantization_id: quantization.id.clone(),
        status: DownloadStatus::Downloading,
        bytes_downloaded: 0,
        total_bytes: Some(quantization.file_size_bytes),
        progress_percent: Some(0.0),
        file_path: None,
        error: None,
    };

    state.set(&model.id, initial.clone());
    emit_progress(&app, &initial);

    let download_result = download_file(
        &app,
        &state,
        &model.id,
        &quantization.id,
        &quantization.download_url,
        &destination,
        quantization.file_size_bytes,
    )
    .await;

    match download_result {
        Ok(()) => {
            let installed = register_installed_model(
                &model.id,
                &model.name,
                &quantization.id,
                &quantization.filename,
                &destination,
            )?;

            let progress = DownloadProgress {
                model_id: model.id.clone(),
                quantization_id: quantization.id.clone(),
                status: DownloadStatus::Completed,
                bytes_downloaded: installed.file_size_bytes,
                total_bytes: Some(installed.file_size_bytes),
                progress_percent: Some(100.0),
                file_path: Some(installed.file_path),
                error: None,
            };

            state.set(&model.id, progress.clone());
            emit_progress(&app, &progress);
            Ok(progress)
        }
        Err(error) => {
            let progress = DownloadProgress {
                model_id: model.id.clone(),
                quantization_id: quantization.id.clone(),
                status: DownloadStatus::Failed,
                bytes_downloaded: 0,
                total_bytes: Some(quantization.file_size_bytes),
                progress_percent: None,
                file_path: None,
                error: Some(error.clone()),
            };

            state.set(&model.id, progress.clone());
            emit_progress(&app, &progress);
            let _ = tokio::fs::remove_file(&destination).await;
            Err(error)
        }
    }
}

async fn download_file(
    app: &AppHandle,
    state: &DownloadState,
    model_id: &str,
    quantization_id: &str,
    url: &str,
    destination: &PathBuf,
    fallback_total: u64,
) -> Result<(), String> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) SelfAPI-Agent/0.1")
        .redirect(reqwest::redirect::Policy::limited(20))
        .connect_timeout(std::time::Duration::from_secs(30))
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .build()
        .map_err(|error| format!("Failed to initialize download client: {}", error))?;

    let response = client
        .get(url)
        .header("Accept", "*/*")
        .send()
        .await
        .map_err(|error| format!("Network request failed for {}: {}", model_id, error))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with status {} for {}. Check internet connectivity or try again.",
            response.status(),
            find_catalog_model(model_id)
                .map(|model| model.name)
                .unwrap_or_else(|| model_id.to_string())
        ));
    }

    let total_bytes = response.content_length().or(Some(fallback_total));
    let mut stream = response.bytes_stream();

    if let Some(parent) = destination.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }

    let mut file = File::create(destination)
        .await
        .map_err(|error| format!("Failed to create destination file {:?}: {}", destination, error))?;

    let mut bytes_downloaded = 0_u64;
    let mut last_emit_bytes = 0_u64;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| format!("Error receiving download stream: {}", error))?;
        file.write_all(&chunk)
            .await
            .map_err(|error| format!("Disk write error: {}", error))?;
        bytes_downloaded += chunk.len() as u64;

        if bytes_downloaded - last_emit_bytes > 512 * 1024 || bytes_downloaded == total_bytes.unwrap_or(0) {
            last_emit_bytes = bytes_downloaded;
            let progress = DownloadProgress {
                model_id: model_id.to_string(),
                quantization_id: quantization_id.to_string(),
                status: DownloadStatus::Downloading,
                bytes_downloaded,
                total_bytes,
                progress_percent: compute_percent(bytes_downloaded, total_bytes),
                file_path: None,
                error: None,
            };

            state.set(model_id, progress.clone());
            emit_progress(app, &progress);
        }
    }

    file.flush().await.map_err(|error| format!("Failed to flush file to disk: {}", error))?;
    Ok(())
}


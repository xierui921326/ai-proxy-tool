// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use tauri_plugin_opener;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub listen: Option<String>,
    pub tls_listen: Option<String>,
    pub openai_upstream: Option<String>,
    pub anthropic_upstream: Option<String>,
}

#[tauri::command]
async fn start_proxy(cfg: Option<UiConfig>) -> Result<(), String> {
    let mut s = core_gateway::settings::Settings::load(None).map_err(|e| e.to_string())?;
    if let Some(c) = cfg {
        if let Some(v) = c.listen { s.listen = v; }
        if let Some(v) = c.tls_listen { s.tls_listen = Some(v); }
        if let Some(v) = c.openai_upstream { s.openai_upstream = Some(v); }
        if let Some(v) = c.anthropic_upstream { s.anthropic_upstream = Some(v); }
        let _ = s.save();
    }
    let ca = core_gateway::ca::CaStore::ensure().map_err(|e| e.to_string())?;
    tauri::async_runtime::spawn(async move {
        let _ = core_gateway::server::run(s, ca).await;
    });
    Ok(())
}

#[tauri::command]
fn apply_hosts() -> Result<(), String> { core_gateway::hosts::apply_default().map_err(|e| e.to_string()) }

#[tauri::command]
fn revert_hosts() -> Result<(), String> { core_gateway::hosts::revert().map_err(|e| e.to_string()) }

#[tauri::command]
fn launch_target(app_path: String, port: u16) -> Result<(), String> { core_gateway::launcher::launch_with_proxy(&app_path, port).map_err(|e| e.to_string()) }

#[tauri::command]
fn load_settings() -> Result<core_gateway::settings::Settings, String> { core_gateway::settings::Settings::load(None).map_err(|e| e.to_string()) }

#[tauri::command]
fn save_settings(cfg: core_gateway::settings::Settings) -> Result<(), String> { cfg.save().map_err(|e| e.to_string()) }

#[tauri::command]
fn export_root_ca_path() -> Result<String, String> {
    let ca = core_gateway::ca::CaStore::ensure().map_err(|e| e.to_string())?;
    let (pem, _key) = ca.export_paths();
    Ok(pem.display().to_string())
}

#[tauri::command]
fn clear_data() -> Result<(), String> {
    let dir = core_gateway::settings::Settings::config_dir();
    std::fs::remove_dir_all(dir).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            start_proxy,
            apply_hosts,
            revert_hosts,
            launch_target,
            load_settings,
            save_settings,
            export_root_ca_path,
            clear_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

mod ai;
mod commands;
mod db;
mod error;
mod models;
mod qpdf;

use std::sync::Arc;

use commands::AppState;
use db::Storage;
use qpdf::QpdfEngine;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_local_data_dir()
                .map_err(|error| format!("无法确定本地数据目录：{error}"))?;
            let storage = Storage::initialize(data_dir)
                .map_err(|error| format!("无法初始化本地数据：{error}"))?;
            let qpdf = QpdfEngine::discover(app.handle());
            app.manage(AppState {
                storage: Arc::new(storage),
                qpdf: Arc::new(qpdf),
                app_version: app.package_info().version.to_string(),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_snapshot,
            commands::create_project,
            commands::update_project,
            commands::delete_project,
            commands::duplicate_project,
            commands::import_materials,
            commands::rename_material,
            commands::delete_material,
            commands::save_project_as_template,
            commands::delete_template,
            commands::export_project,
            commands::get_material_preview_path,
            commands::update_settings,
            commands::test_ai_connection,
            commands::analyze_registration_notice,
            commands::cancel_ai_analysis,
        ])
        .run(tauri::generate_context!())
        .expect("error while running 保研材料助手");
}

//! Skrap: the free, open-source PC cleaner for Windows, made for gamers.
//!
//! A thin layer over the engine (`skrap-core`): commands start a scan or a clean on a background
//! thread, and their progress reaches the page as events.

mod commands;
mod jobs;
mod persist;
mod settings;
mod state;

use std::time::Duration;

use tauri::{AppHandle, Manager, WebviewWindowBuilder};
use tauri_plugin_window_state::StateFlags;

use crate::state::AppState;

const MAIN: &str = "main";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // First: a second launch brings the running window back instead.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main(app);
        }))
        .plugin(
            tauri_plugin_window_state::Builder::new()
                // The page shows the window once it has drawn; the frame is fixed by the config.
                .with_state_flags(StateFlags::all() - StateFlags::VISIBLE - StateFlags::DECORATIONS)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            if let Ok(dir) = app.path().app_log_dir() {
                skrap_win::log_to_file(&dir);
            }
            eprintln!("skrap {}", app.package_info().version);
            let handle = app.handle().clone();
            app.manage(AppState::load(&handle)?);

            // Windows are created now (`create: false`): their page calls commands at once.
            for config in &app.config().app.windows {
                WebviewWindowBuilder::from_config(&handle, config)?.build()?;
            }
            // The page shows the window when it is ready, so it never flashes empty. In case it
            // cannot (a broken page), the window comes up anyway.
            let fallback = handle.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_secs(4));
                if let Some(window) = fallback.get_webview_window(MAIN)
                    && !window.is_visible().unwrap_or(true)
                {
                    show_main(&fallback);
                }
            });
            // Old Reserve batches, and a disk running low, are dealt with in the background.
            std::thread::spawn(move || jobs::maintain_reserve(&handle));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::update_settings,
            commands::get_overview,
            commands::get_rules,
            commands::start_scan,
            commands::start_clean,
            commands::cancel_job,
            commands::get_rule_files,
            commands::get_reserve,
            commands::get_reserve_files,
            commands::restore_batch,
            commands::release_batch,
            commands::reveal,
            commands::now,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Skrap");
}

/// Shows, restores and focuses the main window.
fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

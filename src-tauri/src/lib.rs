//! mono·calendar — catálogo de fechas del día que suena (constitución III).
//!
//! Toda la hora es UTC; el render (front) muestra hora local. El scheduler
//! planifica sobre instantes absolutos para sobrevivir DST y reinicios.

pub mod commands;
pub mod ics;
pub mod model;
pub mod notify;
pub mod scheduler;
pub mod store;

use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, _event| {
                    // US-5.4: mostrar/ocultar el widget desde cualquier app.
                    let _ = shortcut;
                    crate::scheduler::toggle_widget(app);
                })
                .build(),
        )
        .setup(|app| {
            // Estado + almacenamiento local (constitución IV).
            let state = store::init(app.handle())?;
            app.manage(state);

            // Ventana: aplicar posición guardada y rebasarla a área de trabajo.
            crate::scheduler::apply_saved_geometry(app.handle());

            // Registrar atajo global (falla suave si otra app lo usa).
            crate::scheduler::register_shortcut(app.handle());

            // Arrancar el planificador de recordatorios (principio III).
            crate::scheduler::spawn(app.handle().clone());

            // Refresco periódico de feeds (FR-006): cada settings.refresh_minutes,
            // con un primer ciclo inmediato al arrancar (usa la última caché si
            // no hay red — constitución III).
            {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    loop {
                        let _ = crate::commands::refresh_all(&handle).await;
                        let minutes = {
                            let state = handle.state::<crate::store::SchedulerState>();
                            let guard = state.settings.lock().unwrap();
                            let m = guard.refresh_minutes;
                            m
                        };
                        tokio::time::sleep(std::time::Duration::from_secs(
                            (minutes as u64).max(5) * 60,
                        ))
                        .await;
                    }
                });
            }

            // Primer refresco de fechas.
            crate::scheduler::initial_refresh(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::list_feeds,
            commands::add_feed,
            commands::update_feed,
            commands::remove_feed,
            commands::refresh_feeds,
            commands::get_agenda,
            commands::test_notification,
            commands::quit_app,
        ])
        .run(tauri::generate_context!())
        .expect("error al arrancar mono·calendar");
}

//! Notificaciones nativas Windows — tauri-plugin-notification (R-08).
//! El front NUNCA dispara toasts de recordatorios: única fuente = scheduler.

use crate::model::ReminderPlan;
use chrono::{DateTime, Local, TimeZone, Utc};
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

/// Inicio real del evento = trigger + offset (instantes absolutos — edge DST).
fn event_start(plan: &ReminderPlan) -> DateTime<Utc> {
    plan.trigger_at_utc + chrono::Duration::minutes(plan.offset_minutes)
}

fn local_hhmm(plan: &ReminderPlan) -> String {
    Local
        .from_utc_datetime(&event_start(plan).naive_utc())
        .format("%H:%M")
        .to_string()
}

/// Cuenta atrás amable: "comienza en 10 min" / "comienza ahora".
fn countdown(plan: &ReminderPlan) -> String {
    let mins = (event_start(plan) - Utc::now()).num_minutes().max(0);
    match mins {
        0 => "comienza ahora".into(),
        1 => "comienza en 1 min".into(),
        m if m < 60 => format!("comienza en {m} min"),
        _ => "comienza pronto".into(),
    }
}

/// Dispara el toast nativo. Falla suave (nunca tumba el scheduler).
/// `_sound`: el sonido del toast lo gestiona el sistema Windows; el toggle
/// settings.sound queda reservado para el motor de alertas propio (T032).
pub fn fire(app: &AppHandle, plan: &ReminderPlan, _sound: bool) {
    let body = format!("{} · {}", local_hhmm(plan), countdown(plan));
    let _ = app
        .notification()
        .builder()
        .title(plan.event_title.clone())
        .body(body)
        .show();
}

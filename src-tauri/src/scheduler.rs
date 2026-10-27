//! Planificador de recordatorios (principio III de la constitución):
//! persistencia-first, dedupe por firma, cancelación reactiva, replay al arrancar.

use crate::model::*;
use crate::store::SchedulerState;
use chrono::{DateTime, Duration, Utc};
use tauri::{AppHandle, Emitter, Manager};

/// Tick de fondo: 1 s barato (entrega dentro de ±1 s para SC-002).
const TICK_SECONDS: u64 = 1;
/// Ventana de planificación (data-model.md): [ahora, ahora+24h).
const LOOKAHEAD_HOURS: i64 = 24;
/// Tolerancia para entregar un plan atrasado antes de marcarlo missed.
const GRACE_MINUTES: i64 = 5;

// ================= Funciones puras (testeables sin ventana) =================

/// Offsets efectivos: global (reminders_offset default 10) + urgente opcional.
pub fn plan_offsets(settings: &WidgetSettings) -> Vec<i64> {
    let mut v = vec![settings.reminder_offset_minutes];
    if let Some(u) = settings.urgent_offset_minutes {
        if u != settings.reminder_offset_minutes {
            v.push(u);
        }
    }
    v
}

/// Motor de planeación puro: recalcular el plan desde los eventos visibles.
///
/// Reglas (data-model.md):
/// - firma existente en `Scheduled` se conserva (un refresh no la re-crea);
/// - `Delivered`/`Missed`/`Cancelled` nunca se re-arma (dedupe FR-008);
/// - `Scheduled` cuyo evento desapareció (eliminado/movido/margen cambiado) se
///   marca `Cancelled` y su firma se devuelve en la lista de cancelaciones;
/// - vencidos fuera de la gracia se expiran a `Missed` sin disparo (US-2.4).
pub fn compute_plan(
    events: &[CalendarEvent],
    settings: &WidgetSettings,
    current: &[ReminderPlan],
    now: DateTime<Utc>,
) -> (Vec<ReminderPlan>, Vec<String>) {
    let horizon_end = now + Duration::hours(LOOKAHEAD_HOURS);
    let offsets = plan_offsets(settings);

    // Firmas que el estado del mundo pide ahora mismo.
    let mut wanted: std::collections::HashSet<String> = std::collections::HashSet::new();
    for ev in events {
        for off in &offsets {
            let trigger = ev.start_utc - Duration::minutes(*off);
            if trigger < now || trigger >= horizon_end {
                continue;
            }
            wanted.insert(signature(&ev.identity, *off));
        }
    }

    let mut next: Vec<ReminderPlan> = current.to_vec();
    let mut to_cancel: Vec<String> = Vec::new();

    // 1) Cancelar scheduled que ya nadie pide.
    for p in next.iter_mut() {
        if p.status == PlanStatus::Scheduled && !wanted.contains(&p.signature) {
            p.status = PlanStatus::Cancelled;
            to_cancel.push(p.signature.clone());
        }
    }

    // 2) Expirar scheduled vencidos más allá de la gracia.
    for p in next.iter_mut() {
        if p.status == PlanStatus::Scheduled
            && now > p.trigger_at_utc + Duration::minutes(GRACE_MINUTES)
        {
            p.status = PlanStatus::Missed;
        }
    }

    // 3) Añadir los que faltan (no entregados/cancelados/vencidos antes).
    for ev in events {
        for off in &offsets {
            let trigger = ev.start_utc - Duration::minutes(*off);
            if trigger < now || trigger >= horizon_end {
                continue;
            }
            let sig = signature(&ev.identity, *off);
            if next.iter().any(|p| p.signature == sig) {
                continue;
            }
            next.push(ReminderPlan {
                signature: sig,
                event_identity: ev.identity.clone(),
                event_title: ev.title.clone(),
                trigger_at_utc: trigger,
                offset_minutes: *off,
                status: PlanStatus::Scheduled,
            });
        }
    }

    (next, to_cancel)
}

/// Entrega los planes cuya hora llegó (marca delivered/missed y devuelve los
/// que deben dispararse).
pub fn advance(plans: &mut [ReminderPlan], now: DateTime<Utc>) -> Vec<ReminderPlan> {
    let mut due = Vec::new();
    for p in plans.iter_mut() {
        if p.status != PlanStatus::Scheduled {
            continue;
        }
        if now >= p.trigger_at_utc {
            if now <= p.trigger_at_utc + Duration::minutes(GRACE_MINUTES) {
                p.status = PlanStatus::Delivered;
                due.push(p.clone());
            } else {
                p.status = PlanStatus::Missed;
            }
        }
    }
    due
}

/// Al arrancar: scheduled vencidos → missed (US-2.4, sin disparo en cascada).
pub fn expire_stale(plans: &mut [ReminderPlan], now: DateTime<Utc>) -> Vec<RecentEntry> {
    let mut missed = Vec::new();
    for p in plans.iter_mut() {
        if p.status == PlanStatus::Scheduled && p.trigger_at_utc < now {
            p.status = PlanStatus::Missed;
            missed.push(RecentEntry {
                kind: "missed".into(),
                at_utc: p.trigger_at_utc,
                title: format!("recordatorio perdido — {}", p.event_title),
            });
        }
    }
    missed
}

/// Re Construye la agenda (eventos por día) desde las cachés de feeds activos.
pub fn collect_active_events(state: &SchedulerState, now: DateTime<Utc>) -> Vec<CalendarEvent> {
    let feeds = state.feeds.lock().unwrap();
    let window_start = now - Duration::hours(1);
    let window_end = now + Duration::hours(LOOKAHEAD_HOURS);
    let mut out = Vec::new();
    for feed in feeds.iter().filter(|f| f.active) {
        let entry = state.store.load_cache(&feed.id);
        for ev in entry.events {
            if ev.start_utc >= window_start && ev.start_utc < window_end {
                out.push(ev);
            }
        }
    }
    // Deduplicación cross-feed: mismo uid + inicio + título ⇒ una entrada.
    out.sort_by(|a, b| (&a.uid, a.start_utc, &a.title).cmp(&(&b.uid, b.start_utc, &b.title)));
    out.dedup_by(|a, b| a.uid == b.uid && a.start_utc == b.start_utc && a.title == b.title);
    out
}

// ================= Integración con Tauri =================

pub fn refresh_plan(app: &AppHandle) {
    let Some(state) = app.try_state::<SchedulerState>() else {
        return;
    };
    let now = Utc::now();
    let events = collect_active_events(&state, now);
    let settings = state.settings.lock().unwrap().clone();
    let mut plans = state.plans.lock().unwrap();
    let (new_plans, _cancelled) = compute_plan(&events, &settings, &plans, now);
    *plans = new_plans;
    drop(plans);
    persist(&state);
}

fn persist(state: &SchedulerState) {
    let plans = state.plans.lock().unwrap();
    let recent = state.recent.lock().unwrap();
    state.store.save_reminders(&plans, &recent).ok();
}

pub fn spawn(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(TICK_SECONDS)).await;
            tick(&app);
        }
    });
}

fn tick(app: &AppHandle) {
    let Some(state) = app.try_state::<SchedulerState>() else {
        return;
    };
    let now = Utc::now();
    let mut plans = state.plans.lock().unwrap();
    let due = advance(&mut plans, now);
    drop(plans);

    if !due.is_empty() {
        let sound = state.settings.lock().unwrap().sound;
        for plan in &due {
            crate::notify::fire(app, plan, sound);
            let mut recent = state.recent.lock().unwrap();
            recent.insert(
                0,
                RecentEntry {
                    kind: "delivered".into(),
                    at_utc: now,
                    title: plan.event_title.clone(),
                },
            );
            recent.truncate(20);
        }
        persist(&state);
    }
}

/// Al arrancar: expira vencidos, carga feeds y remite agenda al front.
pub fn initial_refresh(app: &AppHandle) {
    let Some(state) = app.try_state::<SchedulerState>() else {
        return;
    };
    let now = Utc::now();
    {
        let mut plans = state.plans.lock().unwrap();
        let missed = expire_stale(&mut plans, now);
        let mut recent = state.recent.lock().unwrap();
        for m in missed {
            recent.insert(0, m);
        }
        recent.truncate(20);
    }
    {
        let feeds = state.store.load_feeds();
        *state.feeds.lock().unwrap() = feeds;
    }
    persist(&state);
    refresh_agenda(app);
}

/// Reconstruye la agenda desde caché y la emite al front.
pub fn refresh_agenda(app: &AppHandle) {
    if let Some(state) = app.try_state::<SchedulerState>() {
        let agenda = crate::commands::agenda_from(&state);
        let _ = app.emit("agenda-updated", &agenda);
    }
}

// US-3.1 opacidad: Tauri 2 no expone set_opacity en desktop; en su lugar el
// front la aplica como opacidad CSS del contenido (`applyOpacity` en theme.ts).

/// Aplica posición guardada (clamp defensivo a valores ≥ 0, US-5.1).
pub fn apply_saved_geometry(app: &AppHandle) {
    let Some(state) = app.try_state::<SchedulerState>() else {
        return;
    };
    let settings = state.settings.lock().unwrap().clone();
    if settings.position.x >= 0 && settings.position.y >= 0 {
        if let Some(win) = app.get_webview_window("widget") {
            let _ = win.set_position(tauri::LogicalPosition::new(
                settings.position.x as f64,
                settings.position.y as f64,
            ));
        }
    }
}

/// Muestra/oculta el widget (atajo global, US-5.4).
pub fn toggle_widget(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("widget") {
        if win.is_visible().unwrap_or(false) {
            let _ = win.hide();
        } else {
            let _ = win.show();
            let _ = win.set_focus();
        }
    }
}

/// Registra el atajo global; falla suave si otra app lo usa (US-5.4).
pub fn register_shortcut(app: &AppHandle) {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    let combo = app
        .try_state::<SchedulerState>()
        .map(|s| s.settings.lock().unwrap().global_shortcut.clone())
        .unwrap_or_else(|| "Ctrl+Alt+C".into());
    let _ = app
        .global_shortcut()
        .register(combo.to_lowercase().as_str());
}

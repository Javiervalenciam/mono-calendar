//! Comandos IPC — contracts/ipc-commands.md. Errores como strings es
//! (el front los muestra tal cual); las URLs jamás aparecen íntegras en ellos.

use crate::ics;
use crate::model::*;
use crate::store::SchedulerState;
use chrono::{DateTime, Duration, Local, TimeZone, Utc};
use tauri::{Manager, State};

pub type CmdResult<T> = Result<T, String>;

// ============ helpers internos ============

fn now() -> DateTime<Utc> {
    Utc::now()
}

/// Ventana de expansión: [ayer, hoy+8) — data-model.md.
fn window() -> (DateTime<Utc>, DateTime<Utc>) {
    let today = Local::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
    let start = Local.from_utc_datetime(&today).with_timezone(&Utc) - Duration::days(1);
    (start, start + Duration::days(9))
}

/// Reconstruye la agenda por días locales desde las cachés (US-1) — usada por
/// scheduler::refresh_agenda y por el comando get_agenda.
pub fn agenda_from(state: &SchedulerState) -> Agenda {
    let now = now();
    let events = crate::scheduler::collect_active_events(state, now);
    let (w_start, w_end) = window();

    let feeds = state.feeds.lock().unwrap();
    let any_stale = feeds
        .iter()
        .filter(|f| f.active)
        .any(|f| f.health != Health::Ok)
        || feeds
            .iter()
            .filter(|f| f.active)
            .any(|f| state.store.load_cache(&f.id).fetched_at_utc.is_none());

    let mut days: Vec<AgendaDay> = Vec::new();
    let base = Local::now().date_naive();
    for d in 0..9 {
        let date = base + Duration::days(d);
        days.push(AgendaDay {
            date: date.format("%Y-%m-%d").to_string(),
            events: Vec::new(),
        });
    }
    for ev in events {
        if ev.start_utc < w_start || ev.start_utc >= w_end {
            continue;
        }
        let local_date = Local
            .from_utc_datetime(&ev.start_utc.naive_utc())
            .date_naive()
            .format("%Y-%m-%d")
            .to_string();
        if let Some(day) = days.iter_mut().find(|d| d.date == local_date) {
            day.events.push(ev);
        }
    }
    for day in &mut days {
        day.events.sort_by_key(|e| e.start_utc);
    }
    Agenda {
        days,
        generated_at: now,
        any_stale,
    }
}

async fn fetch_into_cache(state: &SchedulerState, feed: &mut CalendarFeed) {
    let last_modified = feed.last_modified.clone();
    match ics::fetch::fetch_ics(&feed.url, last_modified.as_deref()).await {
        Ok(result) => {
            if result.not_modified {
                feed.health = Health::Ok;
                feed.last_error = None;
            } else {
                let (w_start, w_end) = window();
                let events = ics::pipeline(&result.body, &feed.id, w_start, w_end);
                let entry = CacheEntry {
                    version: 1,
                    fetched_at_utc: Some(Utc::now()),
                    health: Health::Ok,
                    raw: result.body,
                    last_modified: result.last_modified.clone(),
                    events,
                };
                if state.store.save_cache(&feed.id, &entry).is_err() {
                    feed.health = Health::Error;
                    feed.last_error = Some("No se pudo guardar la caché local".into());
                    return;
                }
                feed.health = Health::Ok;
                feed.last_error = None;
                feed.last_modified = result.last_modified;
            }
            feed.last_fetch_ok_at = Some(Utc::now());
        }
        Err(msg) => {
            // HTTP 4xx = problema del feed (error); otro = red (offline).
            feed.health = if msg.contains("HTTP 4") {
                Health::Error
            } else {
                Health::Offline
            };
            feed.last_error = Some(msg);
        }
    }
}

// ============ comandos ============

#[tauri::command]
pub fn get_settings(state: State<'_, SchedulerState>) -> WidgetSettings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
pub fn save_settings(
    app: tauri::AppHandle,
    state: State<'_, SchedulerState>,
    settings: WidgetSettings,
) -> CmdResult<WidgetSettings> {
    let clean = settings.sanitized();

    // Persistir (atómico, principio III).
    state
        .store
        .save_settings(&clean)
        .map_err(|e| format!("No se pudo guardar la configuración: {e}"))?;
    *state.settings.lock().unwrap() = clean.clone();

    // Atajo global (US-5.4) y autoarranque (US-5.5).
    crate::scheduler::register_shortcut(&app);
    apply_autostart(&app, clean.autostart);

    Ok(clean)
}

fn apply_autostart(app: &tauri::AppHandle, enable: bool) {
    use tauri_plugin_autostart::ManagerExt;
    let autostart = app.autolaunch();
    let _ = if enable { autostart.enable() } else { autostart.disable() };
}

#[tauri::command]
pub fn list_feeds(state: State<'_, SchedulerState>) -> Vec<CalendarFeed> {
    state.feeds.lock().unwrap().clone()
}

#[tauri::command]
pub async fn add_feed(
    app: tauri::AppHandle,
    state: State<'_, SchedulerState>,
    name: String,
    url: String,
) -> CmdResult<CalendarFeed> {
    let name = name.trim().to_string();
    if name.is_empty() || name.len() > 60 {
        return Err("El nombre debe tener entre 1 y 60 caracteres".into());
    }
    ics::fetch::validate_url(&url)?;

    {
        let feeds = state.feeds.lock().unwrap();
        if feeds.iter().any(|f| f.url == url.trim()) {
            return Err("Ese calendario ya está añadido".into());
        }
    }

    let id = ulid::Ulid::new().to_string();
    let mut feed = CalendarFeed::new(id, name, url.trim().to_string());
    // Dos tintas, roles: alterna acento (nunca una tercera — principio I).
    feed.accent = {
        let feeds = state.feeds.lock().unwrap();
        if feeds.iter().filter(|f| f.active).count() % 2 == 0 {
            Accent::Cobalt
        } else {
            Accent::Terra
        }
    };

    // Validación real: debe responder un ICS válido (FR-002) antes de guardar.
    let probe = ics::fetch::fetch_ics(&feed.url, None).await;
    let body = match probe {
        Ok(r) => r.body,
        Err(msg) => return Err(msg),
    };
    let (w_start, w_end) = window();
    let events = ics::pipeline(&body, &feed.id, w_start, w_end);

    let entry = CacheEntry {
        version: 1,
        fetched_at_utc: Some(Utc::now()),
        health: Health::Ok,
        raw: body,
        last_modified: None,
        events,
    };
    state
        .store
        .save_cache(&feed.id, &entry)
        .map_err(|e| format!("No se pudo escribir la caché: {e}"))?;
    feed.health = Health::Ok;
    feed.last_fetch_ok_at = Some(Utc::now());

    state.feeds.lock().unwrap().push(feed.clone());
    let feeds_snapshot = state.feeds.lock().unwrap().clone();
    state
        .store
        .save_feeds(&feeds_snapshot)
        .map_err(|e| format!("No se pudo guardar la lista de calendarios: {e}"))?;

    crate::scheduler::refresh_plan(&app);
    crate::scheduler::refresh_agenda(&app);
    Ok(feed)
}

#[tauri::command]
pub fn update_feed(
    app: tauri::AppHandle,
    state: State<'_, SchedulerState>,
    feed: CalendarFeed,
) -> CmdResult<CalendarFeed> {
    let mut feeds = state.feeds.lock().unwrap();
    match feeds.iter_mut().find(|f| f.id == feed.id) {
        Some(slot) => {
            // La URL cambia ⇒ revalidar en el próximo refresco.
            if slot.url != feed.url {
                slot.health = Health::Offline;
            }
            *slot = feed.clone();
        }
        None => return Err("Calendario no encontrado".into()),
    }
    let snapshot = feeds.clone();
    drop(feeds);
    state
        .store
        .save_feeds(&snapshot)
        .map_err(|e| format!("No se pudo guardar: {e}"))?;
    crate::scheduler::refresh_plan(&app);
    crate::scheduler::refresh_agenda(&app);
    Ok(feed)
}

#[tauri::command]
pub fn remove_feed(
    app: tauri::AppHandle,
    state: State<'_, SchedulerState>,
    feed_id: String,
) -> CmdResult<()> {
    // Cancela recordatorios de eventos de este feed (FR-009).
    {
        let mut plans = state.plans.lock().unwrap();
        for p in plans.iter_mut() {
            if p.event_identity.starts_with(&format!("{feed_id}:"))
                && p.status == PlanStatus::Scheduled
            {
                p.status = PlanStatus::Cancelled;
            }
        }
    }
    {
        let mut feeds = state.feeds.lock().unwrap();
        feeds.retain(|f| f.id != feed_id);
        let snapshot = feeds.clone();
        drop(feeds);
        state
            .store
            .save_feeds(&snapshot)
            .map_err(|e| format!("No se pudo guardar: {e}"))?;
    }
    state.store.remove_cache(&feed_id);
    crate::scheduler::refresh_agenda(&app);
    Ok(())
}

#[tauri::command]
pub async fn refresh_feeds(app: tauri::AppHandle) -> CmdResult<Vec<CalendarFeed>> {
    refresh_all(&app).await
}

/// Ciclo de refresco compartido entre el comando manual y el temporizador de
/// fondo (FR-006). Sin locks vivos durante el await (nunca bloquear la red
/// bajo un Mutex — principio II/III).
pub async fn refresh_all(app: &tauri::AppHandle) -> CmdResult<Vec<CalendarFeed>> {
    let state = app.state::<SchedulerState>();
    let ids_urls: Vec<(String, String, Option<String>)> = {
        let feeds = state.feeds.lock().unwrap();
        feeds
            .iter()
            .map(|f| (f.id.clone(), f.url.clone(), f.last_modified.clone()))
            .collect()
    };

    for (id, url, lm) in ids_urls {
        // Snapshot SIN lock vivo durante el await.
        let mut feed = {
            match state.feeds.lock().unwrap().iter().find(|f| f.id == id) {
                Some(f) => f.clone(),
                None => continue,
            }
        };
        feed.url = url;
        feed.last_modified = lm; // If-Modified-Since en la descarga

        fetch_into_cache(&state, &mut feed).await;

        if let Some(target) = state.feeds.lock().unwrap().iter_mut().find(|f| f.id == id) {
            *target = feed;
        }
    }

    let snapshot = state.feeds.lock().unwrap().clone();
    state
        .store
        .save_feeds(&snapshot)
        .map_err(|e| format!("No se pudo guardar: {e}"))?;
    crate::scheduler::refresh_plan(app);
    crate::scheduler::refresh_agenda(app);
    Ok(snapshot)
}

#[tauri::command]
pub fn get_agenda(state: State<'_, SchedulerState>) -> Agenda {
    agenda_from(&state)
}

#[tauri::command]
pub fn test_notification(app: tauri::AppHandle) {
    use crate::scheduler::plan_offsets;
    let state = app.state::<SchedulerState>();
    let off = plan_offsets(&state.settings.lock().unwrap())[0];
    let demo = ReminderPlan {
        signature: "demo".into(),
        event_identity: "demo".into(),
        event_title: "mono·calendar — recordatorio de prueba".into(),
        trigger_at_utc: Utc::now(),
        offset_minutes: off,
        status: PlanStatus::Delivered,
    };
    let sound = state.settings.lock().unwrap().sound;
    crate::notify::fire(&app, &demo, sound);
}

#[tauri::command]
pub fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

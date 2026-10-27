//! T016 — Pruebas del almacenamiento atómico y tolerancia a corrupción (FR-015).

use chrono::Utc;
use mono_calendar_lib::model::*;
use mono_calendar_lib::store::Store;
use std::fs;

fn store() -> (Store, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    (Store::new(dir.path().to_path_buf()), dir)
}

#[test]
fn settings_ida_y_vuelta_con_sanitizado() {
    let (s, _dir) = store();
    let mut cfg = WidgetSettings::default();
    cfg.refresh_minutes = 30;
    cfg.opacity = 0.75;
    s.save_settings(&cfg).unwrap();

    let loaded = s.load_settings();
    assert_eq!(loaded.refresh_minutes, 30);
    assert_eq!(loaded.opacity, 0.75);

    // Sanitizado: valores fuera de rango se acotan (data-model.md).
    let mut bad = WidgetSettings::default();
    bad.refresh_minutes = 9999;
    bad.opacity = 3.0;
    s.save_settings(&bad).unwrap();
    let loaded = s.load_settings();
    assert_eq!(loaded.refresh_minutes, 240);
    assert_eq!(loaded.opacity, 1.0);
}

#[test]
fn settings_corrupto_se_autocura() {
    let (s, _dir) = store();
    fs::write(s.root().join("settings.json"), "{ esto no es json").unwrap();
    assert!(s.file_corrupted("settings.json"));

    // La carga devuelve defaults y el reset deja el archivo válido.
    let loaded = s.load_settings();
    assert_eq!(loaded.reminder_offset_minutes, 10);
    s.save_settings(&loaded).unwrap();
    assert!(!s.file_corrupted("settings.json"));
    assert!(s.file_ok("settings.json"));
}

#[test]
fn feeds_ida_y_vuelta_y_cache_se_borra() {
    let (s, _dir) = store();
    let mut feeds = vec![
        CalendarFeed::new("a".into(), "Personal".into(), "https://x/a.ics".into()),
        CalendarFeed::new("b".into(), "Trabajo".into(), "https://x/b.ics".into()),
    ];
    feeds[1].health = Health::Ok;
    s.save_feeds(&feeds).unwrap();

    let loaded = s.load_feeds();
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[1].name, "Trabajo");
    assert_eq!(loaded[1].health, Health::Ok);

    // Caché: guardar → existe → quitarla → ya no existe.
    let entry = CacheEntry::default();
    s.save_cache("a", &entry).unwrap();
    let cached = s.load_cache("a");
    assert_eq!(cached.version, 1);
    s.remove_cache("a");
    assert_eq!(s.load_cache("a").events.len(), 0);
}

#[test]
fn recordatorios_ida_y_vuelta_y_tope_recent() {
    let (s, _dir) = store();
    let plans = vec![ReminderPlan {
        signature: "f1:e1:2026-11-03T15:00:00+00:00@10".into(),
        event_identity: "f1:e1:2026-11-03T15:00:00+00:00".into(),
        event_title: "Demo".into(),
        trigger_at_utc: Utc::now(),
        offset_minutes: 10,
        status: PlanStatus::Scheduled,
    }];
    let recent = vec![RecentEntry {
        kind: "delivered".into(),
        at_utc: Utc::now(),
        title: "Anterior".into(),
    }];
    s.save_reminders(&plans, &recent).unwrap();

    let (p2, r2) = s.load_reminders();
    assert_eq!(p2.len(), 1);
    assert_eq!(p2[0].signature, plans[0].signature);
    assert_eq!(r2.len(), 1);

    // El tope de 20 entradas en recent (data-model.md).
    for i in 0..25 {
        s.push_recent(|_, recent| {
            recent.insert(
                0,
                RecentEntry {
                    kind: "delivered".into(),
                    at_utc: Utc::now(),
                    title: format!("n{i}"),
                },
            );
        })
        .unwrap();
    }
    let (_, r3) = s.load_reminders();
    assert_eq!(r3.len(), 20);
}

#[test]
fn archivo_inexistente_da_defaults_limpios() {
    let (s, _dir) = store();
    assert!(!s.file_corrupted("feeds.json"));
    assert!(!s.file_ok("feeds.json"));
    assert!(s.load_feeds().is_empty());
    let (plans, recent) = s.load_reminders();
    assert!(plans.is_empty() && recent.is_empty());
}

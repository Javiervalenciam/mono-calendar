//! T015 — Pruebas del planificador (US-2, FR-007/008/009, plan transitions).

use chrono::{Duration, TimeZone, Utc};
use mono_calendar_lib::model::*;
use mono_calendar_lib::scheduler::{advance, collect_active_events, compute_plan, expire_stale, plan_offsets};
use mono_calendar_lib::store::{SchedulerState, Store};
use std::sync::Mutex;

fn base_now() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 3, 12, 0, 0).unwrap()
}

fn ev(uid: &str, start: chrono::DateTime<Utc>, title: &str) -> CalendarEvent {
    CalendarEvent {
        identity: format!("f1:{uid}:{}", start.to_rfc3339()),
        feed_id: "f1".into(),
        uid: uid.into(),
        title: title.into(),
        start_utc: start,
        end_utc: start + Duration::hours(1),
        all_day: false,
        location: None,
        description: None,
        is_recurring: false,
        series_id: Some(format!("f1:{uid}")),
    }
}

#[test]
fn offsets_default_y_urgente() {
    let s = WidgetSettings::default();
    assert_eq!(plan_offsets(&s), vec![10]);
    let mut s2 = s.clone();
    s2.urgent_offset_minutes = Some(60);
    assert_eq!(plan_offsets(&s2), vec![10, 60]);
}

#[test]
fn plan_crea_y_conserva() {
    let now = base_now();
    let start = now + Duration::minutes(30);
    let events = vec![ev("e1", start, "Demo")];
    let settings = WidgetSettings::default();

    let (plans, cancels) = compute_plan(&events, &settings, &[], now);
    assert!(cancels.is_empty());
    assert_eq!(plans.len(), 1);
    assert_eq!(plans[0].status, PlanStatus::Scheduled);
    // Trigger = instante absoluto inicio − margen (nunca matemática de pared).
    assert_eq!(plans[0].trigger_at_utc, start - Duration::minutes(10));
    assert_eq!(plans[0].event_title, "Demo");

    // Segundo compute idéntico: no duplica ni re-arma.
    let (plans2, _) = compute_plan(&events, &settings, &plans, now);
    assert_eq!(plans2.iter().filter(|p| p.status == PlanStatus::Scheduled).count(), 1);
    assert_eq!(plans2[0].signature, plans[0].signature);
}

fn initial_plans() -> (chrono::DateTime<Utc>, Vec<CalendarEvent>, Vec<ReminderPlan>) {
    let now = base_now();
    let events = vec![ev("e1", now + Duration::minutes(30), "Demo")];
    let (plans, _) = compute_plan(&events, &WidgetSettings::default(), &[], now);
    (now, events, plans)
}

#[test]
fn entregado_no_se_repite() {
    let (now, events, mut plans) = initial_plans();
    let due = advance(&mut plans, now + Duration::minutes(20));
    assert_eq!(due.len(), 1);
    assert_eq!(plans[0].status, PlanStatus::Delivered);

    let (plans2, _) = compute_plan(&events, &WidgetSettings::default(), &plans, now);
    assert!(!plans2.iter().any(|p| p.status == PlanStatus::Scheduled));
}

#[test]
fn evento_eliminado_cancela() {
    let (now, _events, plans) = initial_plans();
    let (plans2, cancels) = compute_plan(&[], &WidgetSettings::default(), &plans, now);
    assert_eq!(cancels.len(), 1);
    assert_eq!(plans2[0].status, PlanStatus::Cancelled);
}

#[test]
fn evento_movido_reprograma() {
    let (now, _events, plans) = initial_plans();
    // El evento reaparece con OTRO inicio (nueva identidad).
    let moved = vec![ev("e1", now + Duration::hours(2), "Demo")];
    let (plans2, cancels) = compute_plan(&moved, &WidgetSettings::default(), &plans, now);
    assert_eq!(cancels.len(), 1); // la firma vieja queda cancelada
    let scheduled: Vec<_> = plans2.iter().filter(|p| p.status == PlanStatus::Scheduled).collect();
    assert_eq!(scheduled.len(), 1);
    assert_eq!(
        scheduled[0].trigger_at_utc,
        moved[0].start_utc - Duration::minutes(10)
    );
    // Instante absoluto correcto: trigger atado a start_utc, no a pared.
}

#[test]
fn vencido_fuera_de_gracia_implica_missed() {
    let now = base_now();
    // Plan cuya hora de disparo fue hace 30 min: fuera de la tolerancia de 5.
    let start = now - Duration::minutes(20);
    let stale = vec![ev("e1", start, "Pasado")];
    let (plans, _) = compute_plan(&stale, &WidgetSettings::default(), &[], now);
    // No puede haber plan para un evento cuyo trigger (start-10min) ya pasó.
    assert!(plans.is_empty());

    // En cambio, si el plan existía de antes (programado pre-corte):
    let pre = ReminderPlan {
        signature: format!("f1:old:{}", (now - Duration::hours(1)).to_rfc3339()),
        event_identity: "f1:old:x".into(),
        event_title: "Viejo".into(),
        trigger_at_utc: now - Duration::minutes(30),
        offset_minutes: 10,
        status: PlanStatus::Scheduled,
    };
    let missed = expire_stale(&mut [pre], now);
    assert_eq!(missed.len(), 1);
    assert_eq!(missed[0].kind, "missed");
}

#[test]
fn gracia_entrega_dentro_de_5_min() {
    let now = base_now();
    let mut p = ReminderPlan {
        signature: "f1:x:y@10".into(),
        event_identity: "f1:x:y".into(),
        event_title: "Con gracia".into(),
        trigger_at_utc: now - Duration::minutes(3),
        offset_minutes: 10,
        status: PlanStatus::Scheduled,
    };
    let due = advance(&mut [p.clone()], now);
    assert_eq!(due.len(), 1); // entregado pese al retraso pequeño

    let mut p2 = p.clone();
    p2.trigger_at_utc = now - Duration::minutes(6);
    let due2 = advance(&mut [p2.clone()], now);
    assert!(due2.is_empty()); // fuera de gracia: missed sin disparo
    p.status = PlanStatus::Delivered;
    let _ = &p;
}

#[test]
fn cambio_de_margen_rota_el_plan() {
    let (now, events, plans) = initial_plans();
    let mut s30 = WidgetSettings::default();
    s30.reminder_offset_minutes = 30;
    let (plans2, cancels) = compute_plan(&events, &s30, &plans, now);
    assert_eq!(cancels.len(), 1); // firma @10 cancelada
    let scheduled: Vec<_> = plans2.iter().filter(|p| p.status == PlanStatus::Scheduled).collect();
    assert_eq!(scheduled.len(), 1);
    assert_eq!(scheduled[0].offset_minutes, 30);
    assert_eq!(
        scheduled[0].trigger_at_utc,
        events[0].start_utc - Duration::minutes(30)
    );
}

#[test]
fn urgente_genera_dos_planes() {
    let now = base_now();
    let start = now + Duration::minutes(90);
    let events = vec![ev("e1", start, "Previsto")];
    let mut s = WidgetSettings::default();
    s.urgent_offset_minutes = Some(60);
    let (plans, _) = compute_plan(&events, &s, &[], now);
    assert_eq!(plans.len(), 2);
    let offs: Vec<i64> = plans.iter().map(|p| p.offset_minutes).collect();
    assert!(offs.contains(&10) && offs.contains(&60));
}

#[test]
fn sin_ventana_no_planes_lejanos() {
    let now = base_now();
    // Evento a 25 horas: fuera del horizonte de 24 h — no se planifica aún.
    let far = vec![ev("e1", now + Duration::hours(25), "Lejano")];
    let (plans, _) = compute_plan(&far, &WidgetSettings::default(), &[], now);
    assert!(plans.is_empty());
}

#[test]
fn dedupe_cross_feed_por_uid_start_title() {
    let dir = tempfile::tempdir().unwrap();
    let state = SchedulerState {
        store: Store::new(dir.path().to_path_buf()),
        settings: Mutex::new(WidgetSettings::default()),
        feeds: Mutex::new(vec![
            CalendarFeed::new("f1".into(), "Uno".into(), "https://x/1.ics".into()),
            CalendarFeed::new("f2".into(), "Dos".into(), "https://x/2.ics".into()),
        ]),
        plans: Mutex::new(vec![]),
        recent: Mutex::new(vec![]),
    };
    let now = base_now();
    let start = now + Duration::minutes(60);
    for feed in ["f1", "f2"] {
        let e = ev("invitado@google", start, "Invitación repetida");
        let entry = CacheEntry {
            version: 1,
            fetched_at_utc: Some(now),
            health: Health::Ok,
            raw: String::new(),
            last_modified: None,
            events: vec![CalendarEvent { feed_id: feed.into(), identity: format!("{feed}:{}", e.identity), ..e }],
        };
        state.store.save_cache(feed, &entry).unwrap();
    }
    let merged = collect_active_events(&state, now);
    assert_eq!(merged.len(), 1, "dos feeds con el mismo evento uno solo visible");
}

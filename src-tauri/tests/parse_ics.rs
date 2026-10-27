//! T014 — Pruebas del parser + expansión ICS sobre fixtures (US-1, FR-003/004).

use chrono::{Duration, TimeZone, Utc};
use mono_calendar_lib::ics;

fn window(y: i32, m: u32, d: u32, days: i64) -> (chrono::DateTime<Utc>, chrono::DateTime<Utc>) {
    let ws = Utc.with_ymd_and_hms(y, m, d, 0, 0, 0).unwrap();
    (ws, ws + Duration::days(days))
}

const SIMPLE: &str = include_str!("fixtures/simple.ics");
const DAILY: &str = include_str!("fixtures/daily.ics");
const WEEKLY: &str = include_str!("fixtures/weekly-rrule.ics");
const MONTHLY: &str = include_str!("fixtures/monthly-rrule.ics");
const BOGOTA: &str = include_str!("fixtures/tzid-bogota.ics");
const DST: &str = include_str!("fixtures/dst-cross.ics");
const EXDATE: &str = include_str!("fixtures/exdate.ics");

#[test]
fn simple_y_all_day() {
    let (ws, we) = window(2026, 11, 1, 9);
    let events = ics::pipeline(SIMPLE, "feed-1", ws, we);
    assert_eq!(events.len(), 2);

    let ev = events.iter().find(|e| e.uid == "simple-1@mono").unwrap();
    assert_eq!(ev.title, "Reunion de equipo");
    assert_eq!(
        ev.start_utc,
        Utc.with_ymd_and_hms(2026, 11, 3, 15, 0, 0).unwrap()
    );
    assert_eq!(
        ev.end_utc,
        Utc.with_ymd_and_hms(2026, 11, 3, 16, 0, 0).unwrap()
    );
    assert!(!ev.all_day);
    assert_eq!(ev.location.as_deref(), Some("Meet"));
    assert!(ev.description.as_deref().unwrap().contains("Revision semanal"));
    assert!(ev.identity.starts_with("feed-1:simple-1@mono:"));
    assert!(!ev.is_recurring);

    let allday = events.iter().find(|e| e.uid == "allday-1@mono").unwrap();
    assert!(allday.all_day);
    assert_eq!(
        allday.start_utc,
        Utc.with_ymd_and_hms(2026, 11, 4, 0, 0, 0).unwrap()
    );
}

#[test]
fn diario_count_cinco() {
    let (ws, we) = window(2026, 11, 1, 9);
    let events = ics::pipeline(DAILY, "feed-1", ws, we);
    assert_eq!(events.len(), 5);
    // Ordenado y sin identidades repetidas.
    let mut ids: Vec<&str> = events.iter().map(|e| e.identity.as_str()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 5);
    assert!(events.windows(2).all(|w| w[0].start_utc <= w[1].start_utc));
    assert_eq!(events[0].start_utc, Utc.with_ymd_and_hms(2026, 11, 2, 9, 0, 0).unwrap());
    assert_eq!(events[4].start_utc, Utc.with_ymd_and_hms(2026, 11, 6, 9, 0, 0).unwrap());
    assert!(events.iter().all(|e| e.is_recurring));
}

#[test]
fn semanal_byday_dos_en_ventana() {
    let (ws, we) = window(2026, 11, 1, 8); // [Nov 1, Nov 9)
    let events = ics::pipeline(WEEKLY, "feed-1", ws, we);
    assert_eq!(events.len(), 2); // mar 3 y jue 5; el mar 10 cae fuera
    assert_eq!(events[0].start_utc, Utc.with_ymd_and_hms(2026, 11, 3, 9, 0, 0).unwrap());
    assert_eq!(events[1].start_utc, Utc.with_ymd_and_hms(2026, 11, 5, 9, 0, 0).unwrap());
    assert!(events.iter().all(|e| e.series_id.as_deref() == Some("feed-1:weekly-1@mono")));
}

#[test]
fn mensual_una_en_ventana() {
    let (ws, we) = window(2026, 11, 1, 8);
    let events = ics::pipeline(MONTHLY, "feed-1", ws, we);
    assert_eq!(events.len(), 1); // la master (5 oct) queda fuera; Nov 5 dentro
    assert_eq!(events[0].start_utc, Utc.with_ymd_and_hms(2026, 11, 5, 18, 0, 0).unwrap());
}

#[test]
fn tzid_bogota_resuelve_a_utc() {
    let (ws, we) = window(2026, 11, 1, 9);
    let events = ics::pipeline(BOGOTA, "feed-1", ws, we);
    assert_eq!(events.len(), 1);
    // 10:00 Bogotá (UTC-5) == 15:00 UTC.
    assert_eq!(events[0].start_utc, Utc.with_ymd_and_hms(2026, 11, 3, 15, 0, 0).unwrap());
    assert_eq!(events[0].end_utc, Utc.with_ymd_and_hms(2026, 11, 3, 16, 0, 0).unwrap());
}

#[test]
fn dst_madrid_hora_pared() {
    // 03:00 hora Madrid el 26-oct-2026 (ya en CET, UTC+1) == 02:00 UTC.
    let (ws, we) = window(2026, 10, 25, 8);
    let events = ics::pipeline(DST, "feed-1", ws, we);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].start_utc, Utc.with_ymd_and_hms(2026, 10, 26, 2, 0, 0).unwrap());
}

#[test]
fn exdate_omite_ocurrencia() {
    let (ws, we) = window(2026, 11, 1, 15); // [Nov 1, Nov 16)
    let events = ics::pipeline(EXDATE, "feed-1", ws, we);
    // Los lunes serían Nov 2, 9, 16; el 9 está excluido y el 16 cae en el borde.
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].start_utc, Utc.with_ymd_and_hms(2026, 11, 2, 9, 0, 0).unwrap());
}

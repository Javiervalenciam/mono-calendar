//! Expansión de recurrencias (RRULE subset de Google) — ver research.md
//! amendment. Soporta FREQ=DAILY|WEEKLY|MONTHLY con INTERVAL, UNTIL, COUNT y
//! BYDAY semanal. Todo sobre instantes UTC (edge DST de la spec).

use super::parse::ParsedFeed;
use crate::model::CalendarEvent;
use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc, Weekday};
use std::str::FromStr;

const MAX_OCCURRENCES: usize = 560; // límite defensivo (principio II)

#[derive(Debug, Clone, Copy, PartialEq)]
enum Freq {
    Daily,
    Weekly,
    Monthly,
}

struct ParsedRule {
    freq: Freq,
    interval: i64,
    until: Option<DateTime<Utc>>,
    count: Option<i64>,
    bydays: Vec<Weekday>,
}

fn parse_weekday(s: &str) -> Option<Weekday> {
    match s.trim().to_uppercase().as_str() {
        "MO" => Some(Weekday::Mon),
        "TU" => Some(Weekday::Tue),
        "WE" => Some(Weekday::Wed),
        "TH" => Some(Weekday::Thu),
        "FR" => Some(Weekday::Fri),
        "SA" => Some(Weekday::Sat),
        "SU" => Some(Weekday::Sun),
        _ => None,
    }
}

fn parse_rule(rrule: &str) -> Option<ParsedRule> {
    let mut freq = None;
    let mut interval = 1i64;
    let mut until = None;
    let mut count = None;
    let mut bydays = Vec::new();

    for part in rrule.split(';') {
        let (k, v) = part.split_once('=')?;
        match k.trim().to_uppercase().as_str() {
            "FREQ" => {
                freq = match v.to_uppercase().as_str() {
                    "DAILY" => Some(Freq::Daily),
                    "WEEKLY" => Some(Freq::Weekly),
                    "MONTHLY" => Some(Freq::Monthly),
                    _ => None,
                };
            }
            "INTERVAL" => interval = i64::from_str(v.trim()).unwrap_or(1).max(1),
            "UNTIL" => {
                until = DateTime::parse_from_str(&format!("{v}+0000"), "%Y%m%dT%H%M%S%z")
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
                    .or_else(|| {
                        chrono::NaiveDate::parse_from_str(v, "%Y%m%d")
                            .ok()
                            .and_then(|d| d.and_hms_opt(23, 59, 59))
                            .map(|ndt| Utc.from_utc_datetime(&ndt))
                    });
            }
            "COUNT" => count = i64::from_str(v.trim()).ok(),
            "BYDAY" => bydays = v.split(',').filter_map(parse_weekday).collect(),
            _ => {}
        }
    }
    Some(ParsedRule {
        freq: freq?,
        interval,
        until,
        count,
        bydays,
    })
}

/// Expande todas las series y une los singles dentro de [start, end).
pub fn expand_recurring(
    parsed: &ParsedFeed,
    window_start: DateTime<Utc>,
    window_end: DateTime<Utc>,
) -> Vec<CalendarEvent> {
    let mut out: Vec<CalendarEvent> = Vec::new();

    // Singles (incluye modificaciones RECURRENCE-ID, que prevalecen).
    for ev in &parsed.singles {
        if ev.start_utc >= window_start && ev.start_utc < window_end {
            out.push(ev.clone());
        }
    }

    for s in &parsed.series {
        let Some(rule_text) = &s.rrule else { continue };
        let Some(rule) = parse_rule(rule_text) else {
            // Regla no soportada: usar la ocurrencia master si cae en ventana.
            if s.base.start_utc >= window_start && s.base.start_utc < window_end {
                out.push(s.base.clone());
            }
            continue;
        };

        // Hora de inicio de la serie marcada como modificada (RECURRENCE-ID).
        let overridden: Vec<DateTime<Utc>> = parsed
            .singles
            .iter()
            .filter(|e| e.series_id == s.base.series_id)
            .map(|e| e.start_utc)
            .collect();

        let duration = s.base.end_utc - s.base.start_utc;
        let mut occurrences: Vec<DateTime<Utc>> = Vec::new();
        let mut cursor = s.base.start_utc;

        for _ in 0..MAX_OCCURRENCES {
            if let Some(until) = rule.until {
                if cursor > until {
                    break;
                }
            }
            if let Some(count) = rule.count {
                if occurrences.len() as i64 >= count {
                    break;
                }
            }
            if cursor >= window_end {
                break;
            }
            occurrences.push(cursor);
            cursor = next_occurrence(cursor, &rule, s.base.start_utc);
        }

        for start in occurrences {
            let excluded =
                s.exdates.contains(&start) || overridden.contains(&start);
            if excluded || start < window_start {
                continue;
            }
            let mut ev = s.base.clone();
            ev.identity = format!(
                "{}:{}:{}",
                s.base.feed_id, s.base.uid,
                start.to_rfc3339()
            );
            ev.start_utc = start;
            ev.end_utc = start + duration;
            ev.is_recurring = true;
            out.push(ev);
        }
    }

    // Dedupe determinista por identidad.
    out.sort_by_key(|e| e.identity.clone());
    out.dedup_by(|a, b| a.identity == b.identity);
    out
}

/// Siguiente ocurrencia de la serie a partir de `t`.
fn next_occurrence(t: DateTime<Utc>, rule: &ParsedRule, base: DateTime<Utc>) -> DateTime<Utc> {
    match rule.freq {
        Freq::Daily => t + Duration::days(rule.interval),
        Freq::Weekly => next_weekly(t, &rule.bydays, rule.interval, base),
        Freq::Monthly => next_monthly(t, rule.interval),
    }
}

/// BYDAY semanal con soporte de INTERVAL por semana (alineación con base).
fn next_weekly(
    t: DateTime<Utc>,
    bydays: &[Weekday],
    interval: i64,
    base: DateTime<Utc>,
) -> DateTime<Utc> {
    if bydays.is_empty() {
        return t + Duration::weeks(interval);
    }
    // Itera por días conservando la hora de `t`, respetando semanas múltiplo.
    let mut cand = next_day_with_weekday(t, bydays);
    for _ in 0..8 {
        if week_index_matches(cand, base, interval) {
            return cand;
        }
        cand = next_day_with_weekday(cand, bydays);
    }
    t + Duration::weeks(interval) // fallback defensivo
}

/// Primer día > t cuyo weekday esté en la lista, conservando hora de t.
fn next_day_with_weekday(t: DateTime<Utc>, bydays: &[Weekday]) -> DateTime<Utc> {
    let hour = t.hour();
    let minute = t.minute();
    let second = t.second();
    let mut date = t.date_naive() + Duration::days(1);
    for _ in 0..8 {
        if bydays.contains(&date.weekday()) {
            if let Some(naive) = date.and_hms_opt(hour, minute, second) {
                return Utc.from_utc_datetime(&naive);
            }
        }
        date += Duration::days(1);
    }
    t + Duration::weeks(1)
}

/// Semana alineada a lunes; true si (semana - semana_base) % interval == 0.
fn week_index_matches(cand: DateTime<Utc>, base: DateTime<Utc>, interval: i64) -> bool {
    let monday = |d: DateTime<Utc>| -> i64 {
        let days = d.date_naive().num_days_from_ce() as i64;
        let weekday_offset = d.date_naive().weekday().num_days_from_monday() as i64;
        (days - weekday_offset) / 7
    };
    (monday(cand) - monday(base)) % interval.max(1) == 0
}

fn next_monthly(t: DateTime<Utc>, interval: i64) -> DateTime<Utc> {
    let mut y = t.year();
    let mut m = t.month() as i32 + interval as i32;
    while m > 12 {
        m -= 12;
        y += 1;
    }
    // El día se recorta si el mes destino es más corto (ej. 31 ene → 28/29 feb).
    let day = t.day().min(days_in_month(y, m as u32));
    let naive = chrono::NaiveDate::from_ymd_opt(y, m as u32, day)
        .and_then(|d| d.and_hms_opt(t.hour(), t.minute(), t.second()));
    naive
        .map(|n| Utc.from_utc_datetime(&n))
        .unwrap_or_else(|| t + Duration::days(30 * interval))
}

fn days_in_month(y: i32, m: u32) -> u32 {
    let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
    let first_next =
        chrono::NaiveDate::from_ymd_opt(ny, nm, 1).expect("fecha válida");
    let first =
        chrono::NaiveDate::from_ymd_opt(y, m, 1).expect("fecha válida");
    (first_next - first).num_days() as u32
}

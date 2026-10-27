//! Parser ICS mínimo y testeable (RFC 5545 subset que exporta Google).
//!
//! Desviación documentada de research R-04 (amendment al final de research.md):
//! parser propio en vez del crate `ical` — control total del unfold, cero
//! sorpresas de API y superficie cubierta por fixtures (T014).

use crate::model::CalendarEvent;
use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;
use std::collections::HashMap;

/// Serie recurrente: eventos master + regla + excepciones.
#[derive(Debug, Clone)]
pub struct ParsedSeries {
    pub base: CalendarEvent,
    /// Texto crudo de RRULE (p.ej. "FREQ=WEEKLY;BYDAY=MO,WE").
    pub rrule: Option<String>,
    /// EXDATE resueltas a UTC.
    pub exdates: Vec<DateTime<Utc>>,
}

#[derive(Debug, Default)]
pub struct ParsedFeed {
    /// Eventos sin recurrencia (incluye modificaciones con RECURRENCE-ID).
    pub singles: Vec<CalendarEvent>,
    /// Series con RRULE activa.
    pub series: Vec<ParsedSeries>,
}

#[derive(Debug, Default, Clone)]
struct Prop {
    name: String,
    value: String,
    params: HashMap<String, Vec<String>>, // p.ej. TZID -> ["America/Bogota"]
}

/// Unfold RFC 5545: líneas que comienzan con espacio/tab continúan la anterior.
fn unfold(raw: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for raw_line in raw.lines() {
        if raw_line.starts_with(' ') || raw_line.starts_with('\t') {
            if let Some(last) = out.last_mut() {
                last.push_str(raw_line.trim_start_matches([' ', '\t']));
            }
        } else {
            out.push(raw_line.to_string());
        }
    }
    out
}

fn parse_prop(line: &str) -> Option<Prop> {
    let line = line.trim_end_matches('\r');
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let (name_params, value) = line.split_once(':')?;
    let mut parts = name_params.split(';');
    let name = parts.next()?.trim().to_uppercase();
    let mut params: HashMap<String, Vec<String>> = HashMap::new();
    for p in parts {
        if let Some((k, v)) = p.split_once('=') {
            params
                .entry(k.trim().to_uppercase())
                .or_default()
                .push(v.trim_matches('"').to_string());
        }
    }
    Some(Prop {
        name,
        value: value.to_string(),
        params,
    })
}

/// Convierte DTSTART/DTEND/EXDATE/RECURRENCE-ID a UTC respetando TZID (R-13).
/// Soporta `YYYYMMDDTHHMMSSZ`, `YYYYMMDDTHHMMSS` con TZID y `YYYYMMDD` (all day).
pub fn parse_dt(
    value: &str,
    params: &HashMap<String, Vec<String>>,
) -> Option<(DateTime<Utc>, bool)> {
    let v = value.trim();
    let all_day = v.len() == 8 && !v.contains('T');

    if v.ends_with('Z') && !all_day {
        let dt = DateTime::parse_from_str(
            &format!("{}+0000", &v[..v.len() - 1]),
            "%Y%m%dT%H%M%S%z",
        )
        .ok()?;
        return Some((dt.with_timezone(&Utc), false));
    }

    if all_day {
        let d = chrono::NaiveDate::parse_from_str(v, "%Y%m%d").ok()?;
        return Some((d.and_hms_opt(0, 0, 0)?.and_utc(), true));
    }

    let naive = chrono::NaiveDateTime::parse_from_str(v, "%Y%m%dT%H%M%S").ok()?;
    if let Some(tzids) = params.get("TZID") {
        if let Some(tzid) = tzids.first() {
            if let Ok(tz) = tzid.parse::<Tz>() {
                if let Some(local) = tz.from_local_datetime(&naive).single() {
                    return Some((local.with_timezone(&Utc), false));
                }
            }
        }
    }
    // Sin TZID reconocible: tratamos como UTC (documentado en R-13).
    Some((Utc.from_utc_datetime(&naive), false))
}

fn clean_text(s: &str) -> String {
    s.replace("\\n", " ")
        .replace("\\,", ",")
        .replace("\\;", ";")
        .replace("\\\\", "\\")
}

fn prop_value<'a>(props: &'a [Prop], name: &str) -> Option<&'a str> {
    props
        .iter()
        .find(|p| p.name == name)
        .map(|p| p.value.as_str())
}

/// Extrae singles + series de un calendario crudo.
pub fn parse_ics(raw: &str, feed_id: &str) -> ParsedFeed {
    let lines = unfold(raw);
    let mut out = ParsedFeed::default();
    let mut in_event = false;
    let mut props: Vec<Prop> = Vec::new();

    for line in lines {
        match line.trim() {
            "BEGIN:VEVENT" => {
                in_event = true;
                props.clear();
            }
            "END:VEVENT" => {
                in_event = false;
                absorb_event(&props, feed_id, &mut out);
            }
            _ if in_event => {
                if let Some(p) = parse_prop(&line) {
                    props.push(p);
                }
            }
            _ => {}
        }
    }
    out
}

fn absorb_event(props: &[Prop], feed_id: &str, out: &mut ParsedFeed) {
    let Some(dtstart) = props
        .iter()
        .find(|p| p.name == "DTSTART")
        .and_then(|p| parse_dt(&p.value, &p.params))
    else {
        return; // sin DTSTART no es un evento útil
    };
    let (start_utc, all_day) = dtstart;

    let uid = prop_value(props, "UID").unwrap_or("no-uid").to_string();
    let end_utc = props
        .iter()
        .find(|p| p.name == "DTEND")
        .and_then(|p| parse_dt(&p.value, &p.params))
        .map(|(dt, _)| dt)
        .unwrap_or_else(|| {
            if all_day {
                start_utc + chrono::Duration::days(1)
            } else {
                start_utc + chrono::Duration::hours(1)
            }
        });

    let title = prop_value(props, "SUMMARY")
        .map(clean_text)
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| "(sin título)".into());

    let ev = CalendarEvent {
        identity: format!("{feed_id}:{uid}:{}", start_utc.to_rfc3339()),
        feed_id: feed_id.to_string(),
        uid: uid.clone(),
        title,
        start_utc,
        end_utc,
        all_day,
        location: prop_value(props, "LOCATION").map(clean_text),
        description: prop_value(props, "DESCRIPTION").map(clean_text),
        is_recurring: false,
        series_id: Some(format!("{feed_id}:{uid}")),
    };

    if let Some(rrule) = prop_value(props, "RRULE") {
        let exdates = props
            .iter()
            .filter(|p| p.name == "EXDATE")
            .filter_map(|p| parse_dt(&p.value, &p.params))
            .map(|(dt, _)| dt)
            .collect();
        let mut modified = ev.clone();
        modified.is_recurring = true; // la serie en sí es recurrente
        out.series.push(ParsedSeries {
            rrule: Some(rrule.to_uppercase()),
            exdates,
            base: modified,
        });
    } else if prop_value(props, "RECURRENCE-ID").is_some() {
        // Modificación de una ocurrencia: entra como single y su serie base
        // se gestiona aparte (la expansión respeta el overriding).
        let mut single = ev;
        single.is_recurring = true;
        out.singles.push(single);
    } else {
        out.singles.push(ev);
    }
}

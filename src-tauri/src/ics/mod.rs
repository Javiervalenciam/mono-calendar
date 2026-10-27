pub mod expand;
pub mod fetch;
pub mod parse;

use expand::expand_recurring;
use parse::{parse_ics, ParsedFeed};
use crate::model::CalendarEvent;
use chrono::{DateTime, Utc};

/// Pipeline completo de un feed: descargar → parsear → expandir a ventana.
pub fn pipeline(
    raw: &str,
    feed_id: &str,
    window_start: DateTime<Utc>,
    window_end: DateTime<Utc>,
) -> Vec<CalendarEvent> {
    let parsed: ParsedFeed = parse_ics(raw, feed_id);
    let mut events = expand_recurring(&parsed, window_start, window_end);
    events.sort_by_key(|e| e.start_utc);
    events
}

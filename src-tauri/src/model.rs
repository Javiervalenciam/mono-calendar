//! Entidades del dominio — contracts/storage-files.md (data-model.md).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Health {
    Ok,
    Error,
    Offline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Accent {
    Cobalt,
    Terra,
}

/// Un calendario de Google conectado vía URL ICS secreta.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CalendarFeed {
    pub id: String,
    pub name: String,
    pub url: String,
    pub active: bool,
    pub health: Health,
    pub last_error: Option<String>,
    pub last_fetch_ok_at: Option<DateTime<Utc>>,
    pub last_modified: Option<String>,
    pub accent: Accent,
}

impl CalendarFeed {
    pub fn new(id: String, name: String, url: String) -> Self {
        Self {
            id,
            name,
            url,
            active: true,
            health: Health::Offline,
            last_error: None,
            last_fetch_ok_at: None,
            last_modified: None,
            accent: Accent::Cobalt,
        }
    }
}

/// Ocurrencia concreta (expandida) de un evento — identidad estable para dedupe.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CalendarEvent {
    /// feed_id:uid:occurrence_start_utc
    pub identity: String,
    pub feed_id: String,
    pub uid: String,
    pub title: String,
    pub start_utc: DateTime<Utc>,
    pub end_utc: DateTime<Utc>,
    pub all_day: bool,
    pub location: Option<String>,
    pub description: Option<String>,
    pub is_recurring: bool,
    pub series_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlanStatus {
    Scheduled,
    Delivered,
    Cancelled,
    Missed,
}

/// Un recordatorio conocido por el planificador.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReminderPlan {
    /// identity + "@" + offset_minutes — firma de dedupe (FR-008)
    pub signature: String,
    pub event_identity: String,
    pub event_title: String,
    pub trigger_at_utc: DateTime<Utc>,
    pub offset_minutes: i64,
    pub status: PlanStatus,
}

/// Firma determinista de un recordatorio (dedupe FR-008).
pub fn signature(event_identity: &str, offset_minutes: i64) -> String {
    format!("{event_identity}@{offset_minutes}")
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Substrate {
    White,
    Gray,
    Beige,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WidgetSize {
    S,
    M,
    L,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WindowPos {
    /// -1 = automático (esquina inferior derecha del área de trabajo)
    pub x: i64,
    pub y: i64,
}

/// Preferencias del usuario — defaults de contracts/storage-files.md.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WidgetSettings {
    pub version: u32,
    pub substrate: Substrate,
    pub size: WidgetSize,
    /// 0.5–1.0 (US-3.1)
    pub opacity: f64,
    pub position: WindowPos,
    pub collapsed: bool,
    /// 5–240 minutos (FR-006)
    pub refresh_minutes: i64,
    /// 0–120 minutos antes (FR-007, default 10)
    pub reminder_offset_minutes: i64,
    /// Aviso adicional temprano opcional (0–120 o null)
    pub urgent_offset_minutes: Option<i64>,
    pub global_shortcut: String,
    pub autostart: bool,
    pub sound: bool,
}

impl Default for WidgetSettings {
    fn default() -> Self {
        Self {
            version: 1,
            substrate: Substrate::White,
            size: WidgetSize::M,
            opacity: 1.0,
            position: WindowPos { x: -1, y: -1 }, // -1 = auto (esquina inf. derecha)
            collapsed: true,
            refresh_minutes: 15,
            reminder_offset_minutes: 10,
            urgent_offset_minutes: None,
            global_shortcut: "Ctrl+Alt+C".into(),
            autostart: false,
            sound: true,
        }
    }
}

impl WidgetSettings {
    /// Validación de rangos de data-model.md; clamp defensivo.
    pub fn sanitized(mut self) -> Self {
        self.opacity = self.opacity.clamp(0.5, 1.0);
        self.refresh_minutes = self.refresh_minutes.clamp(5, 240);
        self.reminder_offset_minutes = self.reminder_offset_minutes.clamp(0, 120);
        if let Some(u) = self.urgent_offset_minutes {
            self.urgent_offset_minutes = Some(u.clamp(0, 120));
        }
        self.refresh_minutes = self.refresh_minutes.max(5);
        self
    }
}

/// Entrada de `recent` (actividad reciente de recordatorios, máx. 20).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RecentEntry {
    pub kind: String, // "delivered" | "missed"
    pub at_utc: DateTime<Utc>,
    pub title: String,
}

/// Caché de eventos expandidos de un feed (8 días).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CacheEntry {
    pub version: u32,
    pub fetched_at_utc: Option<DateTime<Utc>>,
    pub health: Health,
    #[serde(default)]
    pub raw: String,
    #[serde(default)]
    pub last_modified: Option<String>,
    #[serde(default)]
    pub events: Vec<CalendarEvent>,
}

impl Default for CacheEntry {
    fn default() -> Self {
        Self {
            version: 1,
            fetched_at_utc: None,
            health: Health::Offline,
            raw: String::new(),
            last_modified: None,
            events: Vec::new(),
        }
    }
}

/// Día de agenda para el front (get_agenda / agenda-updated).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AgendaDay {
    pub date: String, // YYYY-MM-DD en hora local del equipo
    pub events: Vec<CalendarEvent>,
}

/// Agenda completa para el front (get_agenda / evento agenda-updated).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Agenda {
    pub days: Vec<AgendaDay>,
    pub generated_at: DateTime<Utc>,
    pub any_stale: bool,
}

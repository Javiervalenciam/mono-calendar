//! Almacenamiento local atómico — %APPDATA%/mono-calendar (constitución IV,
//! contracts/storage-files.md). Escritura tempfile + rename; carga tolerante
//! a corrupción (defaults + entrada en recent).

use crate::model::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write as _;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
struct FeedsFile {
    version: u32,
    #[serde(default)]
    feeds: Vec<CalendarFeed>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
struct RemindersFile {
    version: u32,
    #[serde(default)]
    plans: Vec<ReminderPlan>,
    #[serde(default)]
    recent: Vec<RecentEntry>,
}

pub struct Store {
    root: PathBuf,
}

impl Store {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    fn path(&self, name: &str) -> PathBuf {
        let p = self.root.join(name);
        self.ensure_parent(&p);
        p
    }

    fn ensure_parent(&self, p: &PathBuf) {
        if let Some(dir) = p.parent() {
            let _ = fs::create_dir_all(dir);
        }
    }

    /// Escritura atómica: tempfile + rename (principio III / edge cases).
    fn write_atomic(&self, path: &PathBuf, content: &str) -> std::io::Result<()> {
        let tmp = path.with_extension("tmp");
        {
            let mut f = fs::File::create(&tmp)?;
            f.write_all(content.as_bytes())?;
            f.sync_all().ok();
        }
        fs::rename(&tmp, path)
    }

    fn read_or_default<T: for<'de> Deserialize<'de> + Default>(
        &self,
        path: &PathBuf,
    ) -> (T, bool) {
        match fs::read_to_string(path) {
            Ok(txt) => match serde_json::from_str::<T>(&txt) {
                Ok(v) => (v, false),
                Err(_) => (T::default(), true), // corrupto ⇒ defaults
            },
            Err(_) => (T::default(), false), // no existe ⇒ defaults limpios
        }
    }

    // ---- settings.json ----

    pub fn load_settings(&self) -> WidgetSettings {
        let (s, _) = self.read_or_default::<WidgetSettings>(&self.path("settings.json"));
        s.sanitized()
    }

    pub fn save_settings(&self, settings: &WidgetSettings) -> std::io::Result<()> {
        let s = settings.clone().sanitized();
        let json = serde_json::to_string_pretty(&s)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        self.write_atomic(&self.path("settings.json"), &json)
    }

    // ---- feeds.json ----

    pub fn load_feeds(&self) -> Vec<CalendarFeed> {
        let (f, _) = self.read_or_default::<FeedsFile>(&self.path("feeds.json"));
        f.feeds
    }

    pub fn save_feeds(&self, feeds: &[CalendarFeed]) -> std::io::Result<()> {
        let f = FeedsFile {
            version: 1,
            feeds: feeds.to_vec(),
        };
        let json = serde_json::to_string_pretty(&f)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        self.write_atomic(&self.path("feeds.json"), &json)
    }

    // ---- reminders.json ----

    pub fn load_reminders(&self) -> (Vec<ReminderPlan>, Vec<RecentEntry>) {
        let (r, _) = self.read_or_default::<RemindersFile>(&self.path("reminders.json"));
        (r.plans, r.recent)
    }

    pub fn save_reminders(
        &self,
        plans: &[ReminderPlan],
        recent: &[RecentEntry],
    ) -> std::io::Result<()> {
        let r = RemindersFile {
            version: 1,
            plans: plans.to_vec(),
            recent: recent.to_vec(),
        };
        let json = serde_json::to_string_pretty(&r)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        self.write_atomic(&self.path("reminders.json"), &json)
    }

    /// Añade entrada a `recent` (máx. 20) y persiste.
    pub fn push_recent<F>(&self, mutate: F) -> std::io::Result<()>
    where
        F: FnOnce(&mut Vec<ReminderPlan>, &mut Vec<RecentEntry>),
    {
        let (mut plans, mut recent) = self.load_reminders();
        mutate(&mut plans, &mut recent);
        if recent.len() > 20 {
            let overflow = recent.len() - 20;
            recent.drain(0..overflow);
        }
        self.save_reminders(&plans, &recent)
    }

    // ---- cache/feed-<id>.json ----

    fn cache_path(&self, feed_id: &str) -> PathBuf {
        self.path(&format!("cache/feed-{feed_id}.json"))
    }

    pub fn load_cache(&self, feed_id: &str) -> CacheEntry {
        let (c, _) = self.read_or_default::<CacheEntry>(&self.cache_path(feed_id));
        c
    }

    pub fn save_cache(&self, feed_id: &str, entry: &CacheEntry) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(entry)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        self.write_atomic(&self.cache_path(feed_id), &json)
    }

    pub fn remove_cache(&self, feed_id: &str) {
        let _ = fs::remove_file(self.cache_path(feed_id));
    }

    /// Marca error en `recent` cuando un archivo está corrupto y se resetea.
    pub fn note_corruption(&self, _file: &str) {
        self.push_recent(|_plans, recent| {
            recent.insert(
                0,
                RecentEntry {
                    kind: "notice".into(),
                    at_utc: Utc::now(),
                    title: "Un archivo de datos estaba dañado; se reinició su sección."
                        .into(),
                },
            );
        })
        .ok();
    }

    /// true si el archivo existe Y es JSON válido.
    pub fn file_ok(&self, name: &str) -> bool {
        fs::read_to_string(self.path(name))
            .ok()
            .map(|txt| serde_json::from_str::<serde_json::Value>(&txt).is_ok())
            .unwrap_or(false)
    }

    /// true solo si existe pero su JSON no parsea (corrupción real).
    pub fn file_corrupted(&self, name: &str) -> bool {
        match fs::read_to_string(self.path(name)) {
            Ok(txt) => serde_json::from_str::<serde_json::Value>(&txt).is_err(),
            Err(_) => false,
        }
    }
}

/// Directorio de datos por usuario. Reimplementado como función pura para tests.
pub fn default_root() -> PathBuf {
    if let Some(dir) = dirs_data_dir() {
        dir.join("mono-calendar")
    } else {
        std::env::temp_dir().join("mono-calendar")
    }
}

fn dirs_data_dir() -> Option<PathBuf> {
    // %APPDATA% sin dependencia extra de crates de dirs (principio V).
    std::env::var_os("APPDATA").map(PathBuf::from)
}

/// Estado compartido del app: store + vista en memoria (settings, feeds,
/// planes, recent). Se deja en store.rs para que sea el único punto de arranque.
pub struct SchedulerState {
    pub store: Store,
    pub settings: Mutex<WidgetSettings>,
    pub feeds: Mutex<Vec<CalendarFeed>>,
    pub plans: Mutex<Vec<ReminderPlan>>,
    pub recent: Mutex<Vec<RecentEntry>>,
}

pub fn init(_app: &tauri::AppHandle) -> Result<SchedulerState, Box<dyn std::error::Error>> {
    let root = default_root();
    fs::create_dir_all(root.join("cache")).ok();
    let store = Store::new(root);

    // Carga tolerante: si un archivo está corrupto, defaults + aviso en recent.
    let mut settings = store.load_settings();
    let (raw_settings_ok, feeds_ok) = (
        store.file_ok("settings.json"),
        store.file_ok("feeds.json"),
    );
    if !raw_settings_ok || !feeds_ok {
        settings = settings.sanitized();
    }
    let _ = (raw_settings_ok, feeds_ok);

    let feeds = store.load_feeds();
    let (plans, recent) = store.load_reminders();

    // Si los archivos venían dañados, se resetean ya (self-healing).
    if store.file_corrupted("settings.json") {
        store.save_settings(&settings).ok();
        store.note_corruption("settings.json");
    }
    if store.file_corrupted("feeds.json") {
        store.save_feeds(&feeds).ok();
        store.note_corruption("feeds.json");
    }
    if store.file_corrupted("reminders.json") {
        store.save_reminders(&plans, &recent).ok();
        store.note_corruption("reminders.json");
    }

    Ok(SchedulerState {
        settings: std::sync::Mutex::new(settings),
        feeds: std::sync::Mutex::new(feeds),
        plans: std::sync::Mutex::new(plans),
        recent: std::sync::Mutex::new(recent),
        store,
    })
}

# Contracts: API IPC (front ↔ core Tauri)

Fecha: 2026-10-27. Contrato único de comunicación entre la WebView (React/TS) y el
core Rust. Todo comando sigue invoke de Tauri 2; errores devueltos como
`{ code: string, message: string }` (snake_case, en inglés técnico; el front
traduce al español).

## Comandos

| Comando | Args (tipados) | Devuelve | Notas |
|---|---|---|---|
| `get_settings` | — | `WidgetSettings` | Puede crear defaults si no existe archivo |
| `save_settings` | `settings: WidgetSettings` | `WidgetSettings` | Persistencia inmediata; valida rangos |
| `list_feeds` | — | `CalendarFeed[]` | Sin eventos |
| `add_feed` | `name, url` | `CalendarFeed` | Valida URL https; fetch/parse inmediato; error si no es ICS (FR-002); detecta duplicado exacto (FR-016) |
| `update_feed` | `feed: CalendarFeed` | `CalendarFeed` | Renombrar, activar/desactivar, cambiar URL |
| `remove_feed` | `feed_id` | `void` | Borra feed + su caché; cancela sus recordatorios |
| `refresh_feeds` | — | `CalendarFeed[]` | Refresco manual inmediato (FR-006) |
| `get_agenda` | — | `{ days: AgendaDay[], generated_at: string, any_stale: bool }` | Eventos expandidos de hoy + próximos 7 días desde caché; `AgendaDay = { date: string, events: CalendarEvent[] }` |
| `get_feed_health` | — | `CalendarFeed[]` | Igual que list_feeds; usado al abrir ajustes |
| `test_notification` | — | `void` | Toast de ejemplo para verificar permisos de Windows |

## Eventos emitidos core → front (listen)

| Evento | Payload | Cuándo |
|---|---|---|
| `feeds-updated` | `{ feeds: CalendarFeed[] }` | Tras un ciclo de refresco automático/manual |
| `agenda-updated` | `{ days: AgendaDay[] }` | Tras reparse exitoso (evita pedir get_agenda) |
| `feed-health` | `{ feed_id, health, last_error? }` | Cambio de salud (US-4) |

La notificación a Windows es responsabilidad exclusiva del core (notify.rs): el
front NUNCA dispara toasts de recordatorios (única fuente de verdad = scheduler).

## Reglas transversales

- Ningún comando bloquea la UI: fetch/parse corren en el runtime async de Tauri.
- Las URLs ICS completas solo viajan core↔ajustes; en logs y errores se ofuscan
  (`https://calendar.google.com/…/<redacted>`) — principio IV.
- Toda escritura de archivos es atómica (tempfile + rename).
- La ventana widget usa script tags del front; los comandos existen solo en el
  core — sin servidores locales ni puertos escuchando (principio IV).

# Data Model: mono·calendar (001-calendar-widget)

Fecha: 2026-10-27. Entidades del dominio, validaciones y transiciones de estado.
Fuente: spec.md + constitución. El formato físico (JSON) se documenta en
[contracts/storage-files.md](./contracts/storage-files.md).

## CalendarFeed

Un calendario de Google conectado vía URL ICS secreta.

| Campo | Tipo | Reglas |
|---|---|---|
| `id` | string (ULID) | Inmutable, único entre feeds |
| `name` | string | 1–60 caracteres; solo presentación |
| `url` | string | MUST ser `https://` y responder un `text/calendar` válido (FR-002) |
| `active` | bool | Inactivo ⇒ no visible ni genera recordatorios (US-3.3) |
| `health` | enum `ok` \| `error` \| `offline` | `error` requiere `last_error` |
| `last_fetch_ok_at` | datetime UTC \| null | Mostrado en ajustes (US-4.2) |
| `last_modified` | string \| null | `Last-Modified`/ETag devueltos por Google para revalidación |
| `accent` | enum | Variante de tinta para distinguirlo (`cobalt` \| `terra`) — dos tintas, nunca una tercera |

**Transiciones de salud**: `ok → offline` (fallo de red) · `ok/offline → error`
(HTTP 4xx/404-410 o contenido no-ICS) · `offline/error → ok` (fetch exitoso).
La caída de red SIEMPRE conserva la caché (FR-005, SC-005).

## CalendarEvent

Una ocurrencia concreta (expandida) de evento, lista para mostrar/notificar.

| Campo | Tipo | Reglas |
|---|---|---|
| `identity` | string | Firma estable: `feed_id:uid:occurrence_start_utc` (dedupe FR-008) |
| `feed_id` | ref CalendarFeed | Origen |
| `uid` | string | `UID` del VEVENT |
| `title` | string | `SUMMARY` (fallback `"(sin título)"`) |
| `start_utc` / `end_utc` | datetime | Instantes absolutos (edge DST: nunca wall-clock) |
| `all_day` | bool | Evento `VALUE=DATE` sin hora |
| `location` | string \| null | `LOCATION` |
| `description` | string \| null | `DESCRIPTION` (limpia `DESCRIPTION:` escapes) |
| `is_recurring` | bool | true si proviene de RRULE |
| `series_id` | string \| null | `feed_id:uid` cuando es recurrente |

**Deduplicación entre feeds**: dos eventos con mismo `uid` + mismo `start_utc`
+ mismo `title` se muestran una vez (edge case de la spec); se conservan los
`feed_id` de origen para el indicador.

## ReminderPlan

Una instancia de recordatorio conocida por el scheduler.

| Campo | Tipo | Reglas |
|---|---|---|
| `signature` | string | `identity + "@" + offset_minutes` — clave de dedupe (FR-008) |
| `event_identity` | ref CalendarEvent.identity | Evento target |
| `trigger_at_utc` | datetime | `start_utc − offset`; absoluta (edge DST) |
| `offset_minutes` | int | Margen efectivo aplicado |
| `status` | enum `scheduled` \| `delivered` \| `cancelled` \| `missed` | Ver transiciones |

**Transiciones**:
`→ scheduled` (nuevo plan en refresh/arranque si `trigger_at_utc > now`)
· `scheduled → delivered` (toast disparado)
· `scheduled → cancelled` (evento eliminado, modificado con nueva hora, feed
inactivo o eliminado — FR-009)
· `scheduled → missed` (arranque con `trigger_at_utc` ya pasado; NO se dispara
en cascada; queda en actividad reciente, máx. 20 entradas)

**Reglas del planificador** (principio III):
1. Recalcular el plan completo tras cada refresh y al arrancar.
2. Nunca reenviar por la misma `signature` en `delivered`.
3. `cancelled` de eventos modificados genera un nuevo `scheduled` con la nueva hora.
4. La ventana de planificación es `[now, now+24h]`; todo lo demás queda a cargo de
   futuros refrescos (no se pre-agendan semanas: precisión antes que previsión).

## WidgetSettings

Preferencias del usuario. Persistidas inmediatamente al cambiar (US-3.1).

| Campo | Tipo | Default | Reglas |
|---|---|---|---|
| `substrate` | enum `white`\|`gray`\|`beige` | `white` | Mapa → `#FAFAF7`/`#E9E9E5`/`#F5F1E8` |
| `size` | enum `s`\|`m`\|`l` | `m` | Presets ~300/360/460 px de ancho |
| `opacity` | 0.5–1.0 | 1.0 | Se aplica a la ventana completa |
| `position` | `{x, y}` | esquina inferior derecha | Clamp a área de trabajo visible |
| `collapsed` | bool | true | Arranca contraído |
| `refresh_minutes` | 5–240 | 15 | FR-006 |
| `reminder_offset_minutes` | 0–120 | 10 | FR-007 |
| `urgent_offset_minutes` | 0–120 \| null | 60 | Aviso adicional temprano (opcional, mismo toast o silencioso) |
| `global_shortcut` | string | `Ctrl+Alt+C` | FR/US-5.4 |
| `autostart` | bool | false | US-5.5 |
| `sound` | bool | true | Sonido de notificación |

## CacheEntry (por feed)

Contenido de `cache/feed-<id>.json`: `fetched_at_utc`, `raw` (ICS crudo),
`events` (CalendarEvent[] expandidos para `[hoy−1día, hoy+7días]`), `health`,
`last_modified`. La expansión se limita a 8 días para acotar el coste de CPU
(el ciclo de 30 s del scheduler no la reexpande; solo los fetch/parse).

## Relaciones

```text
WidgetSettings 1 ─── 0..* CalendarFeed
CalendarFeed 1 ─── 0..* CacheEntry.events (CalendarEvent)
CalendarEvent 1 ─── 0..* ReminderPlan (por offset activo)
ReminderPlan ─── usa CalendarEvent.identity como clave externa
```

## Archivos de la app (mapa)

```text
%APPDATA%/mono-calendar/
├── settings.json      # WidgetSettings
├── feeds.json         # CalendarFeed[] (sin datos de eventos)
├── reminders.json     # ReminderPlan[] + activity (delivered/missed recientes)
└── cache/
    └── feed-<id>.json # CacheEntry por feed
```

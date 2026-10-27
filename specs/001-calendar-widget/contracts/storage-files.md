# Contracts: Archivos de almacenamiento local

Fecha: 2026-10-27. Esquemas JSON de `%APPDATA%/mono-calendar/`. Todos los campos
`*_utc` son RFC 3339 con sufijo Z. Escritura atómica; tolerancia a archivos
corruptos (arranque con defaults + aviso en la actividad reciente).

## settings.json

```json
{
  "version": 1,
  "substrate": "white",
  "size": "m",
  "opacity": 1.0,
  "position": { "x": 1500, "y": 700 },
  "collapsed": true,
  "refresh_minutes": 15,
  "reminder_offset_minutes": 10,
  "urgent_offset_minutes": null,
  "global_shortcut": "Ctrl+Alt+C",
  "autostart": false,
  "sound": true
}
```

## feeds.json

```json
{
  "version": 1,
  "feeds": [
    {
      "id": "01JBCDEFGH",
      "name": "Personal",
      "url": "https://calendar.google.com/calendar/ical/PRIVATE/basic.ics",
      "active": true,
      "health": "ok",
      "last_error": null,
      "last_fetch_ok_at": "2026-10-27T09:15:00Z",
      "last_modified": "Mon, 26 Oct 2026 18:02:11 GMT",
      "accent": "cobalt"
    }
  ]
}
```

## reminders.json

```json
{
  "version": 1,
  "plans": [
    {
      "signature": "01JBCD:abc123:2026-10-27T15:00:00Z@10",
      "event_identity": "01JBCD:abc123:2026-10-27T15:00:00Z",
      "trigger_at_utc": "2026-10-27T14:50:00Z",
      "offset_minutes": 10,
      "status": "scheduled"
    }
  ],
  "recent": [
    { "kind": "delivered", "at_utc": "…", "title": "Reunión de equipo" }
  ]
}
```

`recent` conserva las últimas 20 entradas `delivered|missed` para la actividad
reciente de la UI (edge case "recordatorio perdido al arrancar").

## cache/feed-<id>.json

```json
{
  "version": 1,
  "fetched_at_utc": "2026-10-27T09:15:00Z",
  "health": "ok",
  "last_modified": "Mon, 26 Oct 2026 18:02:11 GMT",
  "raw": "BEGIN:VCALENDAR…",
  "events": [
    {
      "identity": "01JBCD:abc123:2026-10-27T15:00:00Z",
      "feed_id": "01JBCDEFGH",
      "uid": "abc123",
      "title": "Reunión de equipo",
      "start_utc": "2026-10-27T15:00:00Z",
      "end_utc": "2026-10-27T16:00:00Z",
      "all_day": false,
      "location": "Meet",
      "description": null,
      "is_recurring": false,
      "series_id": null
    }
  ]
}
```

## Invariantes transversales

- `version` presente en todos los archivos para futuras migraciones.
- Los archivos NO contienen secretos observables por otros usuarios (el directorio
  es por-perfil de Windows; el contenido pertenece al mismo usuario — principio IV).
- Un JSON corrupto ⇒ reset de ese archivo a defaults + entrada en `recent`; la app
  nunca arranca en blanco total por un archivo dañado (edge case de la spec).

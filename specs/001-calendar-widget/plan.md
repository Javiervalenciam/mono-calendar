# Implementation Plan: Widget de calendario con recordatorios (mono·calendar)

**Branch**: `001-calendar-widget` | **Date**: 2026-10-27 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-calendar-widget/spec.md`

## Summary

Widget de escritorio flotante para Windows que conecta feeds ICS privados de Google
Calendar (solo lectura), muestra la agenda del día/semana en un chip de escritorio
colapsable y dispara notificaciones push nativas de Windows con offsets
configurables, fiables ante cortes de red y reinicios. El diseño aplica el sistema
editorial mono·color (papel + cobalto/terracota, tipografía Fraunces/Space Grotesk/
IBM Plex Mono, halftone y grano dosificados). El enfoque técnico: Tauri 2 (shell
nativo ligero con WebView2) + React/TS/Vite/Tailwind v4 en el front, y un core en
Rust que gestiona fetch+parse ICS, expansión de recurrencias, caché local y el
planificador de recordatorios con persistencia.

## Technical Context

**Language/Version**: Rust 1.8x (stable-msvc) para core · TypeScript 5.x para front

**Primary Dependencies**: tauri 2 · tauri-plugin-notification · tauri-plugin-autostart
· tauri-plugin-global-shortcut · reqwest (rustls) · ical · rrule · chrono/chrono-tz
· serde/serde_json · React 18 · Vite 7 · Tailwind CSS v4 (@tailwindcss/vite) ·
lucide-react · @fontsource (Fraunces variable, Space Grotesk, IBM Plex Mono)

**Storage**: archivos JSON locales en `%APPDATA%/mono-calendar/` (settings.json,
feeds.json, reminders.json, cache/feed-<id>.json). Sin base de datos.

**Testing**: cargo test (core: parsing ICS, rrule, scheduler, dedupe) · frontend:
comprobaciones de compilación/tsc + validación manual guiada por quickstart.md

**Target Platform**: Windows 10/11 x64 (WebView2 runtime presente en el sistema)

**Project Type**: desktop-app (widget flotante always-on-top, ventana sin marco)

**Performance Goals**: arranque ≤ 1.5 s · RAM reposo ≤ 80 MB · CPU reposo 0 %
perceptible · binario ≤ 15 MB · animaciones de UI a 60 fps sin jank

**Constraints**: sin servidores intermedios (HTTPS directo a Google); sin
telemetría; recordatorios fiables offline (caché local); `prefers-reduced-motion`
obligatorio; paleta ≤ 2 tintas con roles asignados

**Scale/Scope**: 1 usuario local · ≤ ~10 feeds · ~200 eventos visibles máx. en 7
días · 5 historias de usuario (P1: ver eventos, recordatorios; P2: personalizar,
salud de conexión; P3: comportamiento de escritorio)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principio | Estado | Evidencia / Notas |
|---|---|---|
| I. Lenguaje mono·color | ✅ PASS | Tokens `@theme` con paper + cobalto/terracota y roles; fuentes vía @fontsource; texturas (grano feTurbulence, halftone con mask, hairlines, RegMark) como utilidades CSS; gate visual en quickstart |
| II. Rendimiento primero | ✅ PASS (con watchpoint) | Tauri+WebView2: 50–80 MB típicos; se vigila contra el objetivo ≤ 80 MB en el gate de rendimiento. Sin polling agresivo (default 15 min, pausa en idle) |
| III. Fiabilidad recordatorios | ✅ PASS | Scheduler en Rust persistence-first (reminders.json), replanificación al arrancar, dedupe determinista, cancelación por cambio/eliminación — cubierto por tests cargadas en tasks |
| IV. Privacidad local-first | ✅ PASS | Solo %APPDATA% local; HTTPS directo (reqwest+rustls); URLs nunca encima del log ni mostradas íntegras |
| V. Simplicidad | ✅ PASS | Sin BD, sin backend, JSON plano; lógica aislada en módulos puros testables |
| VI. Accesibilidad | ✅ PASS | Contraste ink/paper ≥ 4.5:1; estados con texto+indicador; foco visible en ajustes; `prefers-reduced-motion` | 
| VII. Calidad verificable | ✅ PASS | `cargo test` ejecuta parsing/recurrencia/scheduler incl. DST y offline; gate de diseño en checklist de implement |

Post-Phase 1: sin nuevas violaciones. Los puntos de mira (watchpoint) del
principio II se miden en el quickstart y quedan anotados en Complexity Tracking
solo si se superan.

## Project Structure

### Documentation (this feature)

```text
specs/001-calendar-widget/
├── plan.md              # Este archivo
├── research.md          # Decisiones técnicas (Phase 0)
├── data-model.md        # Entidades y estados (Phase 1)
├── quickstart.md        # Guía de validación end-to-end (Phase 1)
├── contracts/           # Contratos IPC + almacenamiento (Phase 1)
└── tasks.md             # Phase 2 (/speckit.tasks)
```

### Source Code (repository root)

```text
src/                          # Frontend (WebView)
├── main.tsx                  # Entry + montaje de fuentes/tokens
├── App.tsx                   # Árbol: Widget ↔ SettingsPanel
├── styles/index.css          # @theme mono·color + utilidades (grano, halftone, reveal)
├── components/
│   ├── Widget.tsx            # Ventana chip/agenda (contraído/expandido)
│   ├── AgendaList.tsx        # Lista de hoy + próximos 7 días (+N más)
│   ├── EventDetail.tsx       # Detalle de evento (expandido)
│   ├── StatusBar.tsx         # Estado feeds (ok/error/sin conexión) + regla de tiempo
│   ├── SettingsPanel.tsx     # Ajustes: feeds, apariencia, recordatorios, sistema
│   └── editorial/            # RegMark, Reveal, Halftone, Grain (primitivas de diseño)
└── lib/
    ├── ipc.ts                # Wrappers tipados de invoke/escucha (contratos/)
    ├── time.ts               # Formateo local (es), rangos de día
    └── theme.ts              # Sustratos y utilidades de tema

src-tauri/                    # Core Rust
├── src/
│   ├── lib.rs                # Setup Tauri: plugins, estado, ventana widget
│   ├── commands.rs           # Comandos IPC (contracts/ipc-commands.md)
│   ├── ics/
│   │   ├── mod.rs
│   │   ├── fetch.rs          # Descarga HTTPS (reqwest rustls, timeout, ETag/Last-Modified)
│   │   ├── parse.rs          # VEVENT/VTIMEZONE → modelo (crate ical)
│   │   └── expand.rs         # RRULE/UNTIL/COUNT + EXDATE + timezones (crate rrule, chrono-tz)
│   ├── store.rs              # Carga/guardado JSON atómico (%APPDATA%)
│   ├── model.rs              # CalendarFeed, CalendarEvent, ReminderPlan, Settings
│   ├── scheduler.rs          # Planificador: offsets, dedupe, cancelación, replay al arrancar
│   └── notify.rs             # Toast nativo Windows (plugin notification)
├── tauri.conf.json           # Ventana widget: transparent, decorations false, always on top
├── capabilities/*.json       # Permisos mínimos por plugin
└── tests/                    # cargo test (fixtures .ics en tests/fixtures/)

tests/                        # Fixtures compartidos y escenarios de scheduler
```

**Structure Decision**: proyecto único de app de escritorio Tauri con el front en
`src/` y el core en `src-tauri/` (estructura estándar de Tauri 2). La lógica pura
(parsing, expansión, scheduling) vive en módulos Rust sin referencia a la ventana
para cumplir el principio V y VII.

## Complexity Tracking

> Sin violaciones de constitución. Tabla vacía según plantilla.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| (ninguna) | — | — |

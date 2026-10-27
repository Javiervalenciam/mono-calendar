# Research: mono·calendar (001-calendar-widget)

Fecha: 2026-10-27. Cada decisión: elección, por qué y alternativas descartadas.

## R-01 · Shell de escritorio

**Decision**: Tauri 2 (WebView2 del sistema en Windows).
**Rationale**: exe mínimo (~5–10 MB), RAM típica 50–80 MB, ventanas sin marco/
transparentes/always-on-top nativas, plugins oficiales para notificaciones,
autostart y atajo global. WebView2 ya está instalado en el equipo (verificado).
**Alternatives**: Electron (150–300 MB RAM, ~100 MB exe — viola principio II);
WinUI 3/WPF (nativos y ligeros pero ecosistema de animación/diseño editorial
mucho más lento de lograr y menus de sistema inflexibles); PWA (recordatorios
no fiables sin servidor de push y sin widget real en escritorio).

## R-02 · Frontend de la ventana

**Decision**: React 18 + TypeScript + Vite 7 + Tailwind CSS v4.
**Rationale**: el diseño mono·color exige un sistema de componentes con estados
coherentes (principio I), transiciones orquestadas y tokens de diseño: Tailwind v4
`@theme` da exactamente eso con CSS final mínimo; Vite compila y
recarga rápido.
**Alternatives**: vanilla TS (rendimiento máximo, pero más código manual para
estado/animación — el coste va al mantenimiento); Svelte/Solid (más ligeros,
paquete pequeño de componentes y menos ejemplos); se queda React por madurez.
El watchpoint de RAM (≤ 80 MB) se mide en el gate (quickstart).

## R-03 · Tipografías e iconos

**Decision**: @fontsource-variable/fraunces + @fontsource/space-grotesk +
@fontsource/ibm-plex-mono (autoalojadas, offline) y lucide-react para iconos.
**Rationale**: el principio II prohíbe dependencia de CDN (offline) y el
 principio I fija las tres familias como identidad. Fontsource subresourcea
solo los subsets usados.
**Alternatives**: Google Fonts CDN (depende de red, flash de FOUT en widget);
iconos SVG artesanales (lucide ya cubre y es tree-shakeable).

## R-04 · Descarga y parsing de ICS

**Decision**: `reqwest` con TLS rustls (fetch con timeout 15 s, User-Agent propio,
reuso de ETag/Last-Modified si Google los devuelve) + crate `ical` (lexer RFC 5545:
unfold de líneas, propiedades, parámetros) para parsear VEVENT/VTIMEZONE.
**Rationale**: parsing manual de ICS arrastra bugs conocidos (line folding,
escaping de comas y punto y coma en valores, TZID). El crate `ical` es pequeño,
sin dependencias pesadas.
**Alternatives**: crate `ics` (orientado a construir, más incómodo de leer);
`icalendar-rs` (válida, algo más pesada); parser artesanal (riesgo alto).

## R-05 · Recurrencia y tiempo

**Decision**: crate `rrule` para expandir RRULE (FREQ/INTERVAL/UNTIL/COUNT/BYDAY/…)
+ `chrono`/`chrono-tz` para instantes absolutos y TZ.
**Rationale**: el punto 3 de US-1 exige ocurrencias exactas semanales/mensuales;
los casos DST y TZ exigen cálculo en instantes UTC (edge cases de la spec).
`rrule` es el estándar de facto en Rust y compatible con los DTSTART de Google.
**Alternatives**: expansión manual (alto riesgo); producir fechas solo desde el
frontend (rompe el principio V — la lógica debe ser testeable sin ventana).

## R-06 · Almacenamiento local

**Decision**: JSON plano en `%APPDATA%/mono-calendar/` con escritura atómica
(archivo temporal + rename). `settings.json` (apariencia/behavior), `feeds.json`
(calendarios), `reminders.json` (plan de recordatorios), `cache/feed-<id>.json`
(eventos expandidos + crudo descargado).
**Rationale**: escala pequeña (≤10 feeds), inspeccionable y respaldable por el
usuario; escritura atómica evita corrupción (edge case de la spec).
**Alternatives**: SQLite (peso + migraciones innecesarias aquí); registro de
Windows (difícil de respaldar/borrar limpio).

## R-07 · Planificador de recordatorios

**Decision**: motor propio en Rust sobre `tokio` (runtime de Tauri): un loop que
cada 30 s recalcula el plan (eventos desde caché dentro de la ventana
[ahora, ahora + look-ahead 24 h] − margen configurado), compara con `reminders.json`
por firma de dedupe (uid + trigger_at), programa los nuevos en `tokio::time::sleep_until`,
cancela los obsoletos y persiste los cambios.
**Rationale**: cumple III (persistencia→replan al arrancar, dedupe, cancelación
por evento eliminado/reprogramado) sin depender del Task Scheduler de Windows.
Un loop de 30 s despierta la CPU de forma imperceptible (principio II: sin
polling de red, solo reloj local).
**Alternatives**: Windows Task Scheduler XML (permisos, instalación por usuario);
setTimeout en el front (muere si la WebView se recarga / ahorro de energía);
crates de jobs (peso innecesario).

## R-08 · Notificaciones nativas

**Decision**: `tauri-plugin-notification` (toasts de Windows 10/11 con AppID propio).
**Rationale**: nativo, sin dependencias extra; título = nombre del evento, cuerpo =
hora + cuenta atrás ("comienza en 10 min"), sonido respetando preferencia del sistema.
**Alternatives**: notificaciones dibujadas dentro del widget (no visibles si el
widget está oculto/otra app fullscreen); WinRT directo (mismo resultado con más código).

## R-09 · Atajo global y autoarranque

**Decision**: `tauri-plugin-global-shortcut` (por defecto Ctrl+Alt+C) y
`tauri-plugin-autostart` (entrada Run de HKCU, sin elevación).
**Rationale**: plugins oficiales de Tauri cubren US-5 completa sin inventar.
**Alternatives**: RegisterHotKey manual vía winapi (más superficie de código); 
carpeta Startup (visible para el usuario pero menos controlado).

## R-10 · Click-through y arrastre

**Decision**: tarea programada del front que hace hit-test del elemento bajo el
cursor: en zonas transparentes del widget llama a `set_ignore_cursor_events(true)`
y `false` en zonas interactivas; arrastre por la barra del chip usando
`start_dragging()` del API de ventana.
**Rationale**: es el patrón estándar de widgets Tauri; mantiene el gesto "de papel
recortado" del diseño sin barras de ventana del sistema.
**Alternatives**: `WS_EX_TRANSPARENT` completo (el widget entero deja de ser
clicable); mover ventana con mouse events artificiales (frágil).

## R-11 · Texturas de diseño (grano, halftone, marcas)

**Decision**: grano global `feTurbulence` como SVG data-URI en `body::after`
(opacity ~0.5, multiply) · halftone con `radial-gradient` repetido + `mask-image`
radial para el fundido · marcas de registro (RegMark) como componente SVG ·
hairlines `border-ink/10` · animación reveal compartida `cubic-bezier(.22,1,.36,1)`
con IntersectionObserver equivalente vía clase `.in`.
**Rationale**: es exactamente la receta documentada del sistema mono·color
(DESIGN - MONOSKILL.md §2.3/§6) — coste cero en runtime (CSS puro, sin JS de
animación en reposo → principio II).
**Alternatives**: canvas/WebGL para texturas (coste CPU/GPU permanente: prohibido
por II); Lottie (peso + complejidad).

## R-12 · Prerrequisitos de build en este equipo

**Decision**: instalar VS Build Tools 2022 (workload VC++ con MSVC v143 + Windows
SDK, `--includeRecommended`) vía winget y Rust stable-msvc vía rustup; WebView2 ya
presente. Node 26 + npm 11 ya presentes.
**Rationale**: requisitos oficiales de Tauri en Windows para enlazar el binario.
**Alternatives**: toolchain GNU (no soportada oficialmente por Tauri/WebView2).

## R-13 · Fuso horario de eventos de Google

**Decision**: los feeds ICS de Google exportan DTSTART con TZID (p.ej.
America/Bogota) o UTC; se resuelve cada TZID con `chrono-tz` contra VTIMEZONE del
propio feed cuando exista, y se normaliza todo a UTC interno + render local.
**Rationale**: cubre los edge cases de DST/TZ de la spec con una sola norma
(UTC interno, hora local en pantalla).
**Alternatives**: confiar en offsets fijos (bug clásico con DST).

---

## Amendment A-01 (2026-10-27) — Parser y expansor ICS propios

R-04 y R-05 preveían los crates `ical`/`rrule`. Durante la implementación se
optó por un **parser y un expansor propios** (`src-tauri/src/ics/parse.rs`,
`src-tauri/src/ics/expand.rs`) sobre el subset exacto de RFC 5545 que exporta
Google Calendar: unfold de líneas, VEVENT, TZID resuelto vía `chrono-tz`,
RRULE `FREQ=DAILY|WEEKLY|MONTHLY` con `INTERVAL`, `UNTIL`, `COUNT` y `BYDAY`
semanal, `EXDATE` y `RECURRENCE-ID` (overriding de ocurrencias).

**Justificación** (governance, constitución §Governance):
1. Control total de la **identidad estable de ocurrencias** (`feed:uid:inicio`)
   que exige el dedupe FR-008 — con `rrule` la generación de identidades por
   ocurrencia es indireta y frágil.
2. Superficie acotada y **cubierta por fixtures** (tasks T014/T018): 7 ICS de
   prueba con asserts de instantes UTC exactos, all-day, EXDATE, TZID y múltiples
   series.
3. **Menos peso binario** y menos superficie de dependencias (principio II).

**Riesgo acotado**: si aparece una regla no soportada (p. ej. BYMONTHDAY
complejo), la serie degrada de forma definida a su ocurrencia master dentro de
ventana. El crate `rrule` queda documentado aquí como alternativa de migración
si la superficie crece.

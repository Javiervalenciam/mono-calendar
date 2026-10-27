# Tasks: Widget de calendario con recordatorios (mono·calendar)

**Input**: Design documents from `/specs/001-calendar-widget/`
**Prerequisites**: plan.md ✅ · spec.md ✅ · research.md ✅ · data-model.md ✅ · contracts/ ✅
**Tests**: Solicitados por la Constitución (principio VII): la lógica del core
(ICS, recurrencia, scheduler, storage) DEBE tener pruebas automatizadas.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: paralelizable (archivos distintos, sin dependencias pendientes)
- **[Story]**: historia de usuario a la que sirve (US1…US5)

## Path Conventions

Proyecto Tauri 2 único: front en `src/` (React+TS), core en `src-tauri/` (Rust),
tests Rust en `src-tauri/tests/` + fixtures en `src-tauri/tests/fixtures/`.

---

## Phase 1: Setup (infraestructura compartida)

- [ ] T001 Scaffold de Tauri 2 en el repositorio: `src-tauri/` (`Cargo.toml`,
      `tauri.conf.json` con ventana `transparent, decorations:false, always_on_top,
      skip_taskbar:false, width 360, height 520`) + `src/` Vite mínimo y script
      `tauri` en `package.json`
- [ ] T002 Declarar dependencias core en `src-tauri/Cargo.toml`: tauri 2,
      tauri-plugin-notification, tauri-plugin-autostart,
      tauri-plugin-global-shortcut, reqwest (rustls-tls, json), ical, rrule,
      chrono, chrono-tz, serde, serde_json, ulid, thiserror
- [ ] T003 [P] Configurar Tailwind CSS v4 (`@tailwindcss/vite`) y `src/styles/index.css`
      con `@theme` mono·color: paper #FAFAF7, mist #E9E9E5, parch #F5F1E8, cobalt
      #2148B8, cobalt-deep #16307F, cobalt-soft #7FA0E8, terra #C65F38, terra-soft
      #E08E68, ink #20242A, lead #5A6070 + fuentes @fontsource-variable/fraunces,
      @fontsource/space-grotesk, @fontsource/ibm-plex-mono
- [ ] T004 [P] Utilidades de diseño editorial en `src/styles/index.css`: grano
      global `body::after` (feTurbulence data-URI, opacity .5, multiply),
      `.halftone`, `.halftone-terra` (radial-gradient + mask), `.reveal`/.in con
      cubic-bezier(.22,1,.36,1), `@media prefers-reduced-motion` que desactiva
      animaciones, `::selection` cobalto/papel
- [ ] T005 [P] Instalar dependencias npm: react, react-dom, lucide-react,
      @fontsource/*, tailwindcss, @tailwindcss/vite, vite, typescript, @tauri-apps/api

---

## Phase 2: Foundational (bloqueante para todas las US) — CRITICAL

- [ ] T006 Definir modelo del dominio en `src-tauri/src/model.rs`: CalendarFeed
      (id ULID, name 1–60, url https-validada, active, health enum ok|error|
      offline, last_fetch_ok_at, last_modified, accent enum cobalt|terra),
      CalendarEvent (identity = feed_id:uid:occurrence_start_utc, title,
      start_utc/end_utc RFC3339-Z, all_day, location, description, is_recurring,
      series_id), ReminderPlan (signature = identity@offset, trigger_at_utc,
      offset_minutes 0–120, status enum scheduled|delivered|cancelled|missed),
      WidgetSettings (valores exactos de contracts/storage-files.md con defaults:
      white/m/1.0/auto-derecha-inferior/collapsed/15/10/null/Ctrl+Alt+C/false/true)
      — todos con Serialize/Deserialize/Clone y version:1
- [ ] T007 Implementar storage atómico en `src-tauri/src/store.rs`: dir
      %APPDATA%/mono-calendar, archivos settings.json/feeds.json/reminders.json/
      cache/feed-<id>.json (esquemas de contracts/storage-files.md), escritura
      tempfile+rename, carga tolerante a corrupción (defaults + entrada recent),
      helpers list/add/update/remove feeds y save/load caché por feed
- [ ] T008 [P] Implementar fetch ICS en `src-tauri/src/ics/fetch.rs`: reqwest
      con TLS rustls, timeout 15 s, User-Agent propio, envío de If-Modified-Since,
      guardado de Last-Modified, ofuscación de URL en cualquier error/log
      (`…/<redacted>`) — principio IV
- [ ] T009 Implementar parser ICS en `src-tauri/src/ics/parse.rs` con crate `ical`:
      unfold de líneas, VEVENT (SUMMARY/DTSTART/DTEND/DTSTAMP/UID/LOCATION/
      DESCRIPTION/RRULE/EXDATE/RECURRENCE-ID), VALUE=DATE ⇒ all_day, TZID y
      VTIMEZONE resueltos con chrono-tz, normalización a UTC
- [ ] T010 Implementar expansión de recurrencia en `src-tauri/src/ics/expand.rs`
      con crate `rrule`: FREQ/INTERVAL/UNTIL/COUNT/BYDAY, respeto de EXDATE y
      RECURRENCE-ID, ventana de expansión [hoy−1, hoy+8), dedupe cross-feed por
      uid+start_utc+title de data-model.md
- [ ] T011 Implementar scheduler en `src-tauri/src/scheduler.rs`: loop tokio de
      30 s que recalcula plan de [ahora, ahora+24 h] desde caché (offset global
      settings.reminder_offset_minutes y opcional urgent), firma de dedupe,
      sleep_until para disparar, transiciones scheduled→delivered/cancelled/missed
      según data-model.md, persistencia de cambios y emisión de notificación vía
      notify.rs — reproduce plan al arrancar (principio III)
- [ ] T012 [P] Notificaciones nativas en `src-tauri/src/notify.rs` con
      tauri-plugin-notification: título = nombre del evento, cuerpo = hora local
      + ("comienza en X min"), sonido según settings.sound
- [ ] T013 Cablear lib.rs: registro de plugins (notification, autostart con
      macos_launcher mínimo, global-shortcut), arranque del scheduler tras setup,
      carga de settings y aplicación de atajo global (handler toggle visibility)
- [ ] T014 [P] Pruebas del parser/expansión en `src-tauri/tests/parse_ics.rs` con
      fixtures `src-tauri/tests/fixtures/`: simple.ics, daily.ics, weekly-rrule.ics,
      monthly-rrule.ics, tzid-bogota.ics, dst-cross.ics, exdate.ics — asserts de
      identidad, instantes UTC exactos, all_day y dedupe
- [ ] T015 [P] Pruebas del scheduler en `src-tauri/tests/scheduler_test.rs`:
      plan inicial, no-duplicado por firma, cancelación por evento eliminado,
      reprogramación por evento movido, vencido al arrancar ⇒ missed sin disparo
      (mock de clock inyectado)
- [ ] T016 [P] Pruebas de storage en `src-tauri/tests/store_test.rs`: escritura
      atómica, archivo corrupto ⇒ defaults + recent, add/remove feed limpia caché
      y cancela recordatorios
- [ ] T017 Puente front: `src/lib/ipc.ts` con wrappers tipados de todos los
      comandos de contracts/ipc-commands.md (get_settings, save_settings,
      list_feeds, add_feed, update_feed, remove_feed, refresh_feeds, get_agenda,
      get_feed_health, test_notification) + listeners feeds-updated,
      agenda-updated, feed-health + `src/lib/time.ts` (formateo es-ES, rangos de
      día, "en X min") + `src/lib/theme.ts` (mapa sustrato→tokens)

**Checkpoint**: `cargo test` verde sin ventana; contratos IPC compilando en TS.

---

## Phase 3: US1 — Ver los eventos en el widget (P1) — MVP

**Goal**: conectar un feed ICS y ver hoy + próximos 7 días en el widget flotante.
**Independent Test**: `npm run tauri dev` → añadir URL real → eventos visibles ≤ 10 s.

### Tests for US1

- [ ] T018 [P] [US1] Fixture end-to-end: parse+expand de weekly-rrule.ics genera
      2 ocurrencias correctas dentro de la ventana de 8 días (extiende tests del
      T014, mismo archivo)

### Implementation for US1

- [ ] T019 [US1] Comandos IPC add_feed/refresh_feeds/get_agenda en
      `src-tauri/src/commands.rs` (firma de contracts/ipc-commands.md; add_feed
      valida https + ICS válido + duplicado exacto; get_agenda agrupa por día
      desde caché y emite agenda-updated)
- [ ] T020 [US1] Componente `src/components/Widget.tsx`: ventana del widget
      (chip contraído con fecha grande Fraunces itálica + nº de eventos; agenda
      expandida Space Grotesk), fondo paper con hairlines ink/10, useState por
      tema (sustrato via theme.ts)
- [ ] T021 [US1] Componente `src/components/AgendaList.tsx`: secciones HOY y
      PRÓXIMOS event, filas con hora IBM Plex Mono tracking-[0.14em] + título,
      estado vacío editorial con CTA "Añadir calendario", recorte "+N más" a lo
      visible
- [ ] T022 [US1] Componente `src/components/editorial/RegMark.tsx` + anotación
      elíptica terra (SVG stroke-dasharray, rotación −7°, float-y pausable) como
      gesto humano único del widget
- [ ] T023 [US1] Integrar listeners en `src/App.tsx`: agenda-updated refresca
      lista; primer arranque sin feeds muestra empty state (FR-018)
- [ ] T024 [US1] Detalle de evento `src/components/EventDetail.tsx` (clic en fila
      → panel con título/hora/ubicación/descripción, botón volver, animación
      reveal ≤ 250 ms)

**Checkpoint**: US-1 demostrable de forma independiente (quickstart paso 1).

---

## Phase 4: US2 — Recordatorios push (P1)

**Goal**: toasts nativos puntuales, sin duplicados, fiables offline/reinicio.
**Independent Test**: evento de prueba a 12 min + margen 10 → toast exacto; refresco no duplica; evento eliminado no dispara.

### Tests for US2

- [ ] T025 [P] [US2] Tests de transiciones en `src-tauri/tests/scheduler_test.rs`
      ya creados en T015 — completar caso "evento movido ⇒ cancelled + nuevo
      scheduled" y "doble refresh consecutivo ⇒ 0 nuevos planes" (marcar FAIL
      antes de tocar scheduler)

### Implementation for US2

- [ ] T026 [US2] Conectar scheduler.rs al ciclo real: refresh actualiza
      reminders.json (nuevos/cancelados) y dispara toasts vía notify.rs a la hora
      programada; delivered/missed se registran en `recent` (máx. 20)
- [ ] T027 [US2] Cancelación reactiva en commands.rs: update_feed/remove_feed y
      settings.reminder_offset_minutes reprograman el plan en el próximo tick
      (≤ 30 s) según FR-009
- [ ] T028 [US2] Gating de ventana: solo eventos `active=true` de feeds activos
      generan planes (US-3.3 se apoya aquí) + marcar `urgent` cuando aplique
      urgent_offset_minutes

**Checkpoint**: US-1 + US-2 operativas; quickstart pasos 1–3 sin duplicados.

---

## Phase 5: US3 — Personalización (P2)

**Goal**: ajustes en vivo y persistentes (apariencia, recordatorios, sistema).
**Independent Test**: cada cambio se refleja al instante y sobrevive al reinicio.

- [ ] T029 [US3] Comandos get_settings/save_settings en commands.rs (validación
      de rangos de data-model.md: opacity 0.5–1.0, refresh 5–240, offset 0–120)
- [ ] T030 [US3] `src/components/SettingsPanel.tsx`: página de ajustes estilo
      editorial (secciones 01 Calendarios / 02 Apariencia / 03 Recordatorios /
      04 Sistema, índice mono terra, reglas hairline), formularios con label y
      estado de error claros
- [ ] T031 [US3] Apariencia en vivo: apply de substrate/size/opacity/position en
      la ventana Tauri (WebviewWindow set_size/set_opacity) + persistencia
      inmediata (US-3.1/3.2)
- [ ] T032 [US3] Sistema: intervalo de refresco (reinicia el loop de scheduler),
      atajo global (re-registro con global-shortcut), autostart toggle
      (tauri-plugin-autostart), sonido on/off

**Checkpoint**: US-1/US-2 + ajustes persistentes (quickstart paso 4).

---

## Phase 6: US4 — Salud y múltiples calendarios (P2)

**Goal**: estado visible por feed, no dependiente solo del color, sin fallos en silencio.
**Independent Test**: feed válido + feed inválido → estados ok/error legibles.

- [ ] T034 [US4] Gestión de feeds en SettingsPanel (T030): listar con nombre,
      estado (ok/error/offline con texto + punto), último refresco, renombrar,
      eliminar (con remove_feed que cancela recordatorios, FR-009)
- [ ] T035 [US4] Indicador de salud global `src/components/StatusBar.tsx` en el
      widget: "ok"/"sin conexión — usando últimos datos hh:mm"/"error en 1
      calendario" con regla de tiempo ink/10 y copys cortos (dosis
      editorial única)
- [ ] T036 [US4] Estados offline/error y recuperación en AgendaList: banner
      fino (no modal), conserva caché (FR-005), nunca toast de error repetitivo

**Checkpoint**: health visible en widget y ajustes (quickstart paso 5).

---

## Phase 7: US5 — Comportamiento de escritorio (P3)

**Goal**: drag, click-through, colapso/expansión, atajo, autoarranque.
**Independent Test**: cada interacción se verifica suelta sobre el escritorio.

- [ ] T038 [US5] Persistencia de posición por arrastre: barra del chip llama
      `start_dragging()`; al soltar, guardar posición en settings.position clamped
      al área de trabajo (US-5.1)
- [ ] T039 [US5] Click-through: hit-test en `src/App.tsx` con
      `set_ignore_cursor_events` en zonas transparentes (US-5.3)
- [ ] T040 [US5] Colapso/expansión animado (height/opacity inline 250 ms,
      cubic-bezier(.22,1,.36,1), reversible, desactivado con reduced-motion)
      (US-5.2)
- [ ] T041 [US5] atajo global ya cableado en T013 + UI para cambiarlo (T032)
      y autoarranque operativo en Windows (US-5.4/5.5)

**Checkpoint**: quickstart paso 6 completo; las 5 US demostrables.

---

## Phase 8: Polish & Cross-Cutting

- [ ] T043 [P] Copys finales en español + microcopy editorial de estados (US de
      diseño: botones dicen qué ocurrirá)
- [ ] T044 Gate de rendimiento (principio II): medir RAM/CPU en reposo 10 min y
      arranque en frío; registrar resultados en quickstart.md; optimizar si RAM
      > 80 MB (lazy imports, reducir fuentes cargadas)
- [ ] T045 Gate de diseño (principio I): checklist §22 del DESIGN - MONO FRONTEND
      sobre el widget real (jerarquía, dosificación, estados, ruptura controlada)
- [ ] T046 Build release: `npm run tauri build` → verificar binario ≤ 15 MB,
      instalador NSIS, sin consola, icono propio (dos círculos cobalto+terracota)
- [ ] T047 Ejecutar quickstart.md completo y marcar gates de cierre

---

## Dependencies & Execution Order

- **Phase 1 → Phase 2**: el scaffold (T001) precede a todo.
- **Phase 2 es bloqueante**: model/store (T006–T007) antes de ics (T008–T010);
  scheduler (T011) necesita ics+store; T014–T016 pueden escribirse antes de sus
  implementaciones (TDD) pero solo pasan al terminarse estas.
- **US order**: US1 (T019–T024) → US2 (T026–T028 depende de scheduler real) →
  US3/US4 (paralelizables entre sí tras US2) → US5 → Polish.
- **Paralelos marcados [P]**: archivos independientes (p.ej. T014/T015/T016 con
  T008–T010 en curso, y todo el bloque de primitivas de diseño T003/T004 con el
  core).

## Implementation Strategy

- **MVP = Fase 1–4** (Setup + Foundational + US1 + US2): producto útil (ver
  eventos + recordatorios fiables).
- Después US3+US4 (personalización y confianza), US5 (vida diaria), gates finales.
- Cada checkpoint = demo independiente; commit por fase.

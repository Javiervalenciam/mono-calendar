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

- [x] T001 Scaffold de Tauri 2 en el repositorio: `src-tauri/` (`Cargo.toml`,
      `tauri.conf.json` con ventana `transparent, decorations:false, always_on_top,
      skip_taskbar:true, width 360, height 560`) + `src/` Vite mínimo y script
      `tauri` en `package.json`
- [x] T002 Declarar dependencias core en `src-tauri/Cargo.toml`: tauri 2,
      tauri-plugin-notification, tauri-plugin-autostart,
      tauri-plugin-global-shortcut, reqwest (rustls-tls), chrono, chrono-tz,
      serde, serde_json, ulid, tokio — **desviación A-01**: sin `ical`/`rrule`/
      `thiserror` (parser y expansor propios, research.md)
- [x] T003 [P] Configurar Tailwind CSS v4 (`@tailwindcss/vite`) y `src/styles/index.css`
      con `@theme` mono·color: paper #FAFAF7, mist #E9E9E5, parch #F5F1E8, cobalt
      #2148B8, cobalt-deep #16307F, cobalt-soft #7FA0E8, terra #C65F38, terra-soft
      #E08E68, ink #20242A, lead #5A6070 + fuentes @fontsource-variable/fraunces,
      @fontsource/space-grotesk, @fontsource/ibm-plex-mono
- [x] T004 [P] Utilidades de diseño editorial en `src/styles/index.css`: grano
      global `body::after` (feTurbulence data-URI, opacity .5, multiply),
      `.halftone`, `.halftone-terra` (radial-gradient + mask), `animate-reveal`
      con cubic-bezier(.22,1,.36,1), `@media prefers-reduced-motion` que desactiva
      animaciones, `::selection` cobalto/papel
- [x] T005 [P] Instalar dependencias npm: react, react-dom, lucide-react,
      @fontsource/*, tailwindcss, @tailwindcss/vite, vite, typescript, @tauri-apps/api

---

## Phase 2: Foundational (bloqueante para todas las US) — CRITICAL

- [x] T006 Definir modelo del dominio en `src-tauri/src/model.rs`: CalendarFeed
      (id ULID, name 1–60, url https-validada, active, health enum ok|error|
      offline, last_fetch_ok_at, last_modified, accent enum cobalt|terra),
      CalendarEvent (identity = feed_id:uid:occurrence_start_utc, title,
      start_utc/end_utc, all_day, location, description, is_recurring,
      series_id), ReminderPlan (signature = identity@offset, trigger_at_utc,
      offset_minutes, status enum scheduled|delivered|cancelled|missed),
      WidgetSettings (contratos con defaults: white/m/1.0/auto/collapsed/15/10/
      null/Ctrl+Alt+C/false/true) — Serialize/Deserialize/Clone y version:1
- [x] T007 Implementar storage atómico en `src-tauri/src/store.rs`: dir
      %APPDATA%/mono-calendar, archivos settings.json/feeds.json/reminders.json/
      cache/feed-<id>.json, escritura tempfile+rename, carga tolerante a
      corrupción (defaults + entrada recent), helpers save/load por entidad
- [x] T008 [P] Implementar fetch ICS en `src-tauri/src/ics/fetch.rs`: reqwest
      con TLS rustls, timeout 15 s, User-Agent propio, envío de If-Modified-Since,
      guardado de Last-Modified, ofuscación de URL en cualquier error (`redact_url`)
- [x] T009 Implementar parser ICS en `src-tauri/src/ics/parse.rs` — **hand-rolled
      (A-01)**: unfold de líneas, VEVENT (SUMMARY/DTSTART/DTEND/DTSTAMP/UID/
      LOCATION/DESCRIPTION/RRULE/EXDATE/RECURRENCE-ID), VALUE=DATE ⇒ all_day,
      TZID resuelto con chrono-tz, normalización a UTC
- [x] T010 Implementar expansión de recurrencia en `src-tauri/src/ics/expand.rs`
      — **hand-rolled (A-01)**: FREQ=DAILY|WEEKLY+BYDAY|MONTHLY, INTERVAL/UNTIL/
      COUNT, EXDATE y RECURRENCE-ID, ventana [hoy−1, hoy+8); dedupe cross-feed
      en scheduler::collect_active_events (uid+start_utc+title)
- [x] T011 Implementar scheduler en `src-tauri/src/scheduler.rs`: tick tokio de
      **1 s** (SC-002 ±1 s, más estricto que los 30 s citados), plan de
      [ahora, ahora+24 h), firma de dedupe, transiciones
      scheduled→delivered/cancelled/missed, persistencia y emisión vía notify.rs,
      replay de plan al arrancar (principio III)
- [x] T012 [P] Notificaciones nativas en `src-tauri/src/notify.rs` con
      tauri-plugin-notification: título = evento, cuerpo = hora local + cuenta
      atrás; el sonido lo gestiona Windows (settings.sound reservado, T032)
- [x] T013 Cablear lib.rs: plugins (notification, autostart, global-shortcut con
      handler toggle), arranque del scheduler, refresco periódico (FR-006),
      posición guardada + atajo global al arrancar
- [x] T014 [P] Pruebas del parser/expansión en `src-tauri/tests/parse_ics.rs`:
      7 fixtures, asserts de identidad, instantes UTC exactos, all_day — **7/7 ok**
- [x] T015 [P] Pruebas del scheduler en `src-tauri/tests/scheduler_test.rs`:
      plan inicial, no-duplicado, cancelación, reprogramación, missed con clock
      inyectado — **11/11 ok**
- [x] T016 [P] Pruebas de storage en `src-tauri/tests/store_test.rs`: escritura
      atómica, corrupción ⇒ defaults, caché por feed, tope recent=20 — **5/5 ok**
- [x] T017 Puente front: `src/lib/ipc.ts` wrappers tipados + listeners
      (agenda-updated, feeds-updated) + `src/lib/time.ts` (es-ES) + `src/lib/theme.ts`

**Checkpoint**: `cargo test` verde sin ventana (23/23); contratos IPC en TS ✅.

---

## Phase 3: US1 — Ver los eventos en el widget (P1) — MVP

**Goal**: conectar un feed ICS y ver hoy + próximos 7 días en el widget flotante.
**Independent Test**: `npm run tauri dev` → añadir URL real → eventos visibles ≤ 10 s.

### Tests for US1

- [x] T018 [P] [US1] Fixture end-to-end: weekly-rrule.ics genera 2 ocurrencias
      en ventana de 8 días (`semanal_byday_dos_en_ventana`, parse_ics.rs)

### Implementation for US1

- [x] T019 [US1] Comandos IPC add_feed/refresh_feeds/get_agenda en commands.rs
      (add_feed valida https + duplicado; get_agenda agrupa por día desde caché
      y emite agenda-updated)
- [x] T020 [US1] `src/components/Widget.tsx`: chip contraído (fecha grande
      Fraunces itálica + nº eventos/próximo) ⇄ agenda expandida, hairlines ink/10
- [x] T021 [US1] `src/components/AgendaList.tsx`: secciones HOY/MAÑANA/día,
      filas hora IBM Plex Mono + título, estado vacío editorial con CTA, "+N más"
- [x] T022 [US1] `src/components/editorial/RegMark.tsx` + anotación elíptica
      terra (SVG stroke-dasharray, float-y) como gesto editorial único
- [x] T023 [US1] Listeners integrados en `src/App.tsx`: agenda-updated refresca;
      primer arranque sin feeds ⇒ empty state (FR-018)
- [x] T024 [US1] Detalle de evento `src/components/EventDetail.tsx` (clic en fila
      → título/hora/ubicación/notas/origen, volver, reveal 260 ms)

**Checkpoint**: US-1 demostrable de forma independiente (quickstart paso 1).

---

## Phase 4: US2 — Recordatorios push (P1)

**Goal**: toasts nativos puntuales, sin duplicados, fiables offline/reinicio.
**Independent Test**: evento de prueba a 12 min + margen 10 → toast exacto; refresco no duplica; evento eliminado no dispara.

### Tests for US2

- [x] T025 [P] [US2] Transiciones cubiertas en scheduler_test.rs: ``evento_movido_reprograma``
      (cancelled + nuevo scheduled) y ``plan_crea_y_conserva`` / ``entregado_no_se_repite``
      (doble compute ⇒ 0 planes nuevos)

### Implementation for US2

- [x] T026 [US2] Scheduler conectado al ciclo real: reminders.json persistido,
      toasts vía notify.rs a la hora programada, delivered/missed en `recent` (≤20)
- [x] T027 [US2] Cancelación reactiva en commands.rs: update_feed/remove_feed y
      cambio de offset reprograman en el próximo tick (≤1 s) — FR-009
- [x] T028 [US2] Gating: solo feeds `active=true` generan planes; offset urgente
      opcional genera segundo plan(`plan_offsets` + `urgente_genera_dos_planes`)

**Checkpoint**: US-1 + US-2 operativas; quickstart pasos 1–3 sin duplicados.

---

## Phase 5: US3 — Personalización (P2)

**Goal**: ajustes en vivo y persistentes (apariencia, recordatorios, sistema).
**Independent Test**: cada cambio se refleja al instante y sobrevive al reinicio.

- [x] T029 [US3] get_settings/save_settings con sanitizado de rangos
      (opacity 0.5–1.0, refresh 5–240, offset 0–120 — probado en store_test)
- [x] T030 [US3] `src/components/SettingsPanel.tsx`: secciones 01 Calendarios /
      02 Apariencia / 03 Recordatorios / 04 Sistema, índice mono terra, reglas
      hairline, errores claros
- [x] T031 [US3] Apariencia en vivo: sustrato/size/opacidad — **desviación Tauri 2**:
      opacidad vía variable CSS `--app-opacity` (no existe `set_opacity` en la
      API/ACL de Tauri 2); size con LogicalSize; persistencia inmediata
- [x] T032 [US3] Sistema: intervalo de refresco (el loop lee settings cada ciclo),
      atajo global re-registrado, autostart toggle, sonido on/off persistido

**Checkpoint**: US-1/US-2 + ajustes persistentes (quickstart paso 4).

---

## Phase 6: US4 — Salud y múltiples calendarios (P2)

**Goal**: estado visible por feed, no dependiente solo del color, sin fallos en silencio.
**Independent Test**: feed válido + feed inválido → estados ok/error legibles.

- [x] T034 [US4] Gestión de feeds en SettingsPanel: listar con estado
      (ok/error/offline con texto + marca no-cromática), renombrar (ENTER/ESC),
      eliminar con confirmación inline (remove_feed cancela recordatorios)
- [x] T035 [US4] Indicador global `src/components/StatusBar.tsx`: SINCRONIZADO /
      SIN CONEXIÓN · ÚLTIMOS DATOS hh:mm / ERROR EN N CALENDARIOS, con marcas
      ■/▲/○ y regla hairline
- [x] T036 [US4] Banner fino en AgendaList cuando `any_stale` (no modal),
      caché conservada (FR-005), nunca toast repetitivo de error

**Checkpoint**: health visible en widget y ajustes (quickstart paso 5).

---

## Phase 7: US5 — Comportamiento de escritorio (P3)

**Goal**: drag, click-through, colapso/expansión, atajo, autoarranque.
**Independent Test**: cada interacción se verifica suelta sobre el escritorio.

- [x] T038 [US5] Posición persistente: header llama `startDragging()`; onMoved
      con debounce 500 ms guarda en settings.position; el core re-aplica y
      rebasa al arrancar (apply_saved_geometry)
- [x] T039 [US5] Click-through: root `pointer-events:none` + superficies
      interactivas `pointer-events:auto`; la ventana es exactamente del tamaño
      del contenido (no hay zonas muertas que tapar)
- [x] T040 [US5] Colapso/expansión con tamaño animado por el SO + reveal del
      contenido 260 ms cubic-bezier(.22,1,.36,1), desactivado con reduced-motion
- [x] T041 [US5] Atajo global cableado (toggle) + campo editable en Ajustes +
      autostart operativo en Windows (plugin autostart)

**Checkpoint**: quickstart paso 6 completo; las 5 US demostrables.

---

## Phase 8: Polish & Cross-Cutting

- [x] T043 [P] Copys finales en español + microcopy editorial de estados
      ("Papel en blanco", "Silencio en la página", "SIN CONEXIÓN · ÚLTIMOS DATOS", …)
- [ ] T044 Gate de rendimiento (principio II): medir RAM/CPU en reposo 10 min y
      arranque en frío; registrar resultados en quickstart.md; optimizar si RAM
      > 80 MB
- [x] T045 Gate de diseño (principio I): checklist §22 — **PASS**, registro en
      checklists/design-gate.md (1 desviación menor corregida: hex→token accent)
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

---

## Estado de ejecución (2026-10-27)

- Fases 1–7: **implementadas** (T001–T043) con las desviaciones documentadas
  (A-01 parser propio; opacidad CSS; tick 1 s).
- `cargo test`: **23/23 verde** (parser 7 · scheduler 11 · storage 5).
- `npm run build` (frontend): OK — 184 kB JS (58 kB gzip), fuentes autoalojadas.
- Pendiente: T044–T047 (gates de rendimiento/diseño, build release, quickstart).

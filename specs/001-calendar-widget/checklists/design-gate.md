# Design Gate — T045 (principio I, DESIGN mono·color §22)

**Fecha**: 2026-10-27 · **Artefacto auditado**: widget real (src/, build TS OK,
cargo test 23/23). El doc maestro «DESIGN - MONO FRONTEND» vive fuera de este
repo; esta auditoría aplica sus criterios §22 sobre el código implementado.

## 1. Un solo sustrato visible

- [x] El widget usa exactamente un sustrato por tema: `--substrate`
  (paper #FAFAF7 / gris #E9E9E5 / beige #F5F1E5) aplicado a `body` y a cada
  tarjeta (`background: var(--substrate)`).
  `src/styles/index.css:35` · `src/components/Widget.tsx` · `SettingsPanel.tsx`.
- [x] Sin grises adicionales «decorativos»: sólo los tokens declarados
  (mist/parch reservados como sustratos alternativos, no como capas).

## 2. ≤ 2 tintas con rol

- [x] Cobalto #2148B8 dominante: header, horas (Plex Mono), labels de sección,
  botón primario solid (`bg-cobalt text-paper`), foco.
- [x] Terracota #C65F38 SOLO acento: banner de desconexión, «+N MÁS», elipse
  editorial, acciones destructivas y CTA de prueba. Sin tercer tono semántico.
- [x] Ink/Lead son neutros de texto, no cuentan como tinta de marca.
- [x] **Corregido en este gate**: 3 checkboxes/slider usaban hex literal
  `accent-[#2148B8]` → migrados a utility del token `accent-cobalt`.

## 3. Texturas dosificadas y prescindibles

- [x] Grano de papel: `body::after` feTurbulence con alpha 0.045, opacity .5,
  multiply, `pointer-events:none` — retirarlo no rompe nada (es decoración pura).
  `src/styles/index.css:47`.
- [x] Halftone: trama radial 0.8px con máscara elíptica y opacidad 0.14–0.18;
  sólo en estados vacíos (nunca sobre datos). `src/styles/index.css:78`.
- [x] Zero skeuomorfismo funcional: borders hairline `ink/10`, sombra única
  suave de elevación.

## 4. Roles tipográficos

- [x] Fraunces (display, itálica): fecha grande del chip, «Hoy» editorial,
  títulos. `src/components/Widget.tsx`, `RegMark.tsx` no aplica.
- [x] Space Grotesk (body): títulos de eventos, descripciones, controles.
- [x] IBM Plex Mono (mono): horas 11px tracking‑8%, labels técnicos
  uppercase tracking‑16‑26%, contadores. Tres roles sin contaminación cruzada.
- [x] Fuentes autoalojadas (R-03): `dist/assets/*woff2` — sin CDN, sin FOUT.

## 5. Movimiento

- [x] `animate-reveal`: 260 ms, cubic-bezier(.22, 1, .36, 1), sin rebotes.
- [x] floaty 3.4 s sólo en la elipse decorativa.
- [x] **prefers-reduced-motion mata todo movimiento** (media query global).
  `src/styles/index.css:100`.

## 6. Estados editoriales (no cromáticos)

- [x] Vacío sin feeds → «Papel en blanco» + CTA (FR-018).
- [x] Vacío con feeds → «Sin eventos» + elipse terracota + nota humana.
- [x] Salud con texto + marca ■/▲/○ además del color (US-4, accesibilidad VI):
  `src/components/StatusBar.tsx`.
- [x] Errores de feed legibles en SettingsPanel (texto, no sólo color).

## 7. Ruptura controlada

- [x] Un solo gesto «imprenta»: RegMark (círculo + retícula) y la elipse
  manuscrita; ambos stroke 1–1.5 px, sin gradientes ni glows.

## Veredicto

**PASS** — la interfaz cumple los 7 criterios del gate; 1 desviación menor
(hex literales en accent) detectada y corregida durante la propia auditoría.
Pendiente de re-verificación visual cuando el usuario ejecute el widget real
(quickstart paso 6) por si la dosis de grano/halftone necesita ajuste fino.

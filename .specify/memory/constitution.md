<!--
SYNC IMPACT REPORT
==================
Version change: (nueva) → 1.0.0
Modified principles: ninguna (ratificación inicial)
Added sections:
  - Core Principles (I–VII)
  - Restricciones Técnicas y de Alcance
  - Flujo de Trabajo y Quality Gates
  - Governance
Removed sections: ninguna
Follow-up TODOs: ninguno
-->

# mono·calendar Constitution

## Core Principles

### I. Lenguaje de diseño mono·color (INNEGOCIABLE)

Toda la interfaz sigue el sistema editorial de dos tintas definido en los documentos de
diseño del proyecto:

- Sustrato de papel visible: `paper #FAFAF7` (o gris frío `#E9E9E5` / beige pálido
  `#F5F1E8` como variantes temáticas). El papel no cuenta como tinta.
- Máximo DOS tintas con roles asignados: **cobalto `#2148B8`** como tinta dominante
  (estructura, jerarquía, 70–85 %) y **terracota `#C65F38`** como tinta de acento
  (acción, índices, alertas, 15–30 %). Nunca una tercera tinta; nunca color sin rol.
- Tipografía con tensión: **Fraunces** (display serif itálica), **Space Grotesk**
  (cuerpo y títulos), **IBM Plex Mono** (metadatos, horas, etiquetas). Salto de escala
  dramático entre display y microtexto (5–12×).
- Texturas editoriales dosificadas: grano de papel sutil, halftone, hairlines de 1 px
  (`ink/10–20`), marcas de registro y anotaciones manuales. La interfaz DEBE seguir
  funcionando si se retiran todos los detalles editoriales.
- Debe aplicarse el Quality Gate visual del documento de diseño antes de dar cada
  funcionalidad por terminada.

### II. Rendimiento primero

El widget debe ser imperceptible para el sistema:

- Memoria objetivo en reposo: ≤ 80 MB. Arranque en frío: ≤ 1.5 s.
- CPU en reposo: 0 % perceptible. Prohibidas animaciones continuas mientras el usuario
  no interactúa (auto-pausa de toda ambient animation en idle).
- Sin polling agresivo: refresco de calendarios con intervalo configurable
  (por defecto 15 min) y refresco manual explícito.
- Cada dependencia nueva DEBE justificarse frente a su coste; prohibido añadir
  frameworks UI pesados o runtimes adicionales.
- Binario final objetivo: ≤ 15 MB.

### III. Fiabilidad de recordatorios (INNEGOCIABLE)

- Toda notificación programada DEBE dispararse: los recordatorios próximos se
  persisten localmente, se reprogranan al arrancar y funcionan con la última caché
  de eventos si no hay red.
- Prohibidas las notificaciones duplicadas: deduplicación determinista por
  identidad de evento + hora de inicio.
- Si un feed ICS falla o expira, el widget DEBE degradarse con el último dato válido
  y un indicador de "sin conexión" — nunca en silencio.
- El sistema de recordatorios no DEPENDE de que el servidor de Google responda:
  solo lectura ICS + caché local.

### IV. Privacidad local-first

- Las URLs ICS y la configuración se guardan SOLO en el equipo del usuario, en el
  directorio de configuración estándar de la app.
- Sin telemetría, sin análisis de terceros, sin servidores intermedios: el tráfico
  es HTTPS directo a los endpoints de Google que el usuario indicó.
- Las URLs ICS son secretas por diseño: nunca se registran en logs ni se exponen
  en la interfaz más allá de lo imprescindible.

### V. Simplicidad y eficiencia

- YAGNI estricto: no construir configuración o features sin evidencia de necesidad.
- Lógica de calendario (parsing ICS, recurrencia, schedule) aislada del render
  para poder probarla sin ventana.
- La ventana del widget sin marco: arrastrable, con click-through en zonas
  transparentes y estados mínimos (contraído/expandido).

### VI. Accesibilidad

- Contraste conforme en paper/ink; nunca depender solo del color para comunicar
  estado (siempre color + texto o posición o icono).
- Foco visible y navegación por teclado en la configuración.
- `prefers-reduced-motion` DEBE respetarse: el movimiento se reduce o elimina.

### VII. Calidad verificable

- La lógica de parsing ICS, recurrencia y planificación de recordatorios DEBE tener
  pruebas automatizadas (se ejecutan en CI local con un solo comando).
- Cada funcionalidad de UI se verifica contra el Quality Gate del documento de
  diseño (jerarquía, estados, dosificación editorial, responsive del widget).

## Restricciones Técnicas y de Alcance

- Stack: **Tauri 2 + Rust** (core, scheduling, notificaciones nativas de Windows) +
  **TypeScript + React + Vite + Tailwind CSS v4** (front). Sin Electron.
- Plataforma objetivo: Windows 10/11 x64. Un solo binario instalable/portable ≤ 15 MB.
- Fuente de datos: feeds ICS privados de Google Calendar (solo lectura), múltiples
  calendarios soportados con color/estado propio por feed.
- Notificaciones: toasts nativos de Windows; recordatorios por offsets personalizables
  (global y por evento puntual).
- Personalización: tamaño y opacidad del widget, sustrato temático (white/gray/beige),
  offsets de recordatorio, calendarios activos, intervalo de refresco, autoarranque.
- UI en español, manteniendo términos técnicos en inglés (ICS, OAuth, halftone).

## Flujo de Trabajo y Quality Gates

- Flujo Spec-Driven Development con Spec Kit: constitution → specify → (clarify) →
  plan → tasks → implement → converge. Ninguna feature se implementa sin spec y plan
  vigentes.
- Gate de rendimiento: medir memoria y CPU en reposo antes de cerrar cada feature;
  registrarlo en las tareas.
- Gate de diseño: revisión contra los 12 principios del documento mono·color frontend
  (superficie, color con propósito, retícula, espacio activo, tipografía, gesto humano,
  estados, motion, dosificación, microcopy).
- Gate de fiabilidad: prueba automatizada del schedule de recordatorios (incluye casos
  límite: evento en curso, modificado/eliminado en Google, DST, reinicio del sistema).

## Governance

- Esta constitución prevalece sobre cualquier otra práctica del repositorio.
- Las enmiendas se documentan con `/speckit.constitution`, incremento semántico de
  versión, fecha de última modificación y informe de impacto sincronizado.
- Toda PR/revisión DEBE verificar conformidad con los principios I–VII; la complejidad
  añadida DEBE justificarse por escrito.
- Use `.specify/memory/` como fuente de principios vigentes y `specs/NNN-*/` como
  fuente de decisiones por feature.

**Version**: 1.0.0 | **Ratified**: 2026-10-27 | **Last Amended**: 2026-10-27

# Feature Specification: Widget de calendario con recordatorios (mono·calendar)

**Feature Branch**: `001-calendar-widget`

**Created**: 2026-10-27

**Status**: Draft

**Input**: User description: "Crear un sistema de widget tipo calendario que se conecte con mi Google Calendar y genere notificaciones push de recordatorios de eventos; personalizable, muy óptimo, que consuma pocos recursos, con diseño moderno, animaciones fluidas y el estilo de diseño mono·color (documentos DESIGN - MONO FRONTEND.md, SKILL - MONO SKILL.md y DESIGN - MONOSKILL.md). Decisión del usuario: app de escritorio Windows (widget flotante); conexión por enlace ICS privado de Google Calendar (solo lectura)."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Ver los eventos del calendario en el widget (Priority: P1)

Como usuario, quiero pegar la dirección privada iCal de mi Google Calendar en los
ajustes del widget y ver inmediatamente los eventos de hoy y de los próximos días
en un widget flotante sobre mi escritorio, con el estilo editorial mono·color.

**Why this priority**: sin visualización de eventos el widget no existe; es el núcleo
del producto y todo lo demás se apoya en él.

**Independent Test**: se configura un feed ICS de prueba y el widget muestra los
eventos correctamente, sin conexión a ningún otro componente.

**Acceptance Scenarios**:

1. **Given** una configuración vacía (primer arranque), **When** el usuario abre los
   ajustes, añade un nombre y una URL ICS válida y guarda, **Then** el widget muestra
   los eventos de hoy y los próximos 7 días del calendario conectado en ≤ 10 s.
2. **Given** un calendario conectado, **When** el widget se refresca (auto o manual),
   **Then** los eventos nuevos, modificados o eliminados en Google se reflejan en el
   widget en el siguiente ciclo de refresco.
3. **Given** un feed con eventos recurrentes (semanales, mensuales), **When** se
   muestran los próximos 7 días, **Then** las ocurrencias recurrentes aparecen en su
   fecha y hora correctas según su regla.
4. **Given** un evento con título, hora, ubicación y descripción, **When**
   hago clic en él en el widget, **Then** se muestra su detalle completo en el panel
   expandido.

---

### User Story 2 - Recordatorios push antes de los eventos (Priority: P1)

Como usuario, quiero recibir una notificación push del sistema antes de cada evento
(según un margen configurable) para no olvidar mis reuniones, fiable aunque haya
perdido conexión o haya reiniciado el equipo.

**Why this priority**: el propósito expreso del producto es no olvidar eventos; es la
segunda mitad del núcleo y define la confianza en el producto.

**Independent Test**: se crea un evento de prueba a pocos minutos y se verifica que
la notificación nativa aparece al momento programado, sin duplicados.

**Acceptance Scenarios**:

1. **Given** un evento que empieza en 30 min y el margen global configurado de
   "10 minutos antes", **When** falta exactamente 10 minutos para el inicio, **Then**
   se dispara una notificación nativa de Windows con título, hora y nombre del evento.
2. **Given** una notificación ya mostrada para un evento, **When** el widget refresca
   y el evento sigue existiendo sin cambiar, **Then** NO se muestra una segunda
   notificación para el mismo recordatorio.
3. **Given** un evento eliminado en Google después de haberse programado su
   recordatorio, **When** llega la hora programada, **Then** no se dispara ninguna
   notificación.
4. **Given** el equipo reiniciado, **When** el widget arranca y existe un recordatorio
   cuya hora programada ya pasó o pasa durante la sesión actual, **Then** los
   recordatorios futuros se reprograman y los vencidos no se disparan en cascada
   (máximo un resumen de perdidos, si aplica).
5. **Given** el widget sin conexión a internet, **When** llega la hora de un
   recordatorio basado en la caché local de eventos, **Then** la notificación se
   dispara igualmente.

---

### User Story 3 - Personalizar el widget (Priority: P2)

Como usuario, quiero ajustar tamaño, opacidad, tema de sustrato, posición, márgenes
de recordatorio, calendarios activos e intervalo de refresco, para que el widget se
adapte a mi escritorio y a mi forma de trabajar.

**Why this priority**: la personalización fue un requisito explícito del usuario y
diferencia al producto, pero aporta valor solo sobre un núcleo funcional.

**Independent Test**: cambiando cada ajuste desde el panel de configuración se
observa el efecto inmediato y persistente tras reabrir la app.

**Acceptance Scenarios**:

1. **Given** el panel de ajustes, **When** cambio el tamaño del widget entre sus
   presets y la opacidad, **Then** la ventana aplica los cambios en vivo y los
   recuerda tras cerrar y reabrir.
2. **Given** tres temas de sustrato disponibles (papel blanco, gris frío, beige
   pálido), **When** selecciono uno, **Then** todas las superficies del widget usan
   ese sustrato sin romper el contraste de las dos tintas.
3. **Given** varios calendarios conectados, **When** desactivo uno, **Then** sus
   eventos dejan de mostrarse y de generar recordatorios sin eliminar su configuración.
4. **Given** márgenes de recordatorio configurables, **When** defino "30 minutos
   antes" como margen global, **Then** todos los recordatorios nuevos usan ese margen
   hasta que lo cambie.
5. **Given** un intervalo de refresco configurado, **When** trascurre dicho intervalo,
   **Then** el widget actualiza su caché de eventos una única vez y vuelve a reposo.

---

### User Story 4 - Salud de la conexión y múltiples calendarios (Priority: P2)

Como usuario, quiero conectar varios calendarios de Google y saber en todo momento si
los feeds están sanos, para distinguir "no tengo eventos" de "mi calendario no se
puede leer".

**Why this priority**: evita el fallo silencioso (principio III de la constitución)
y soporta el caso real de calendarios separados (trabajo, personal).

**Independent Test**: con dos feeds configurados (uno válido, uno inválido) el widget
muestra el estado de cada uno con un indicador no-dependiente-del-color.

**Acceptance Scenarios**:

1. **Given** dos feeds configurados, uno con URL revocada, **When** se refresca,
   **Then** el widget muestra los eventos del feed válido y marca el roto con un
   estado visible (texto + indicador) sin borrar su configuración.
2. **Given** una pestaña de ajustes, **When** reviso la lista de calendarios, **Then**
   veo nombre, estado (ok / error / sin conexión), último refresco exitoso y acciones
   (renombrar, refrescar, eliminar).
3. **Given** sin internet, **When** el widget intenta refrescar, **Then** usa la
   caché local y muestra "sin conexión — usando últimos datos" con la marca de
   tiempo del último refresco exitoso.

---

### User Story 5 - Comportamiento de escritorio (Priority: P3)

Como usuario, quiero arrastrar el widget a donde quiera, colapsarlo/expanderlo,
usar un atajo de teclado para ocultarlo/mostrarlo, hacer clic a través de sus zonas
transparentes y opcionalmente que arranque con Windows.

**Why this priority**: mejora la vida diaria con el widget pero no define el valor
central; puede completarse después de P1/P2.

**Independent Test**: cada interacción se verifica por separado sobre el escritorio
con el widget ejecutándose.

**Acceptance Scenarios**:

1. **Given** el widget visible, **When** lo arrastro por su zona de barra, **Then**
   queda anclado en la nueva posición y la recuerda tras reiniciar la app.
2. **Given** el widget en modo contraído (solo chip del día), **When** hago clic,
   **Then** se expande con una transición fluida y reversible (≤ 250 ms), continuando
   la posición del contenido.
3. **Given** zonas transparentes del widget, **When** hago clic sobre ellas, **Then**
   el clic atraviesa hacia las ventanas y el escritorio por debajo.
4. **Given** un atajo de teclado configurable (por defecto Ctrl+Alt+C), **When** lo
   pulso desde cualquier app, **Then** el widget se oculta/muestra.
5. **Given** autoarranque habilitado en ajustes, **When** reinicio sesión en Windows,
   **Then** el widget arranca minimizado a su última posición sin ventana de consola.

---

### Edge Cases

- ¿Qué pasa si la URL ICS se introduce mal o no es HTTPS? → validación con mensaje
  concreto antes de guardar; nunca se guarda una URL inválida sin aviso.
- ¿Qué pasa si el usuario elimina el calendario en Google (el feed deja de existir)?
  → estado "error 410/404" visible; la caché permanece como solo-lectura con aviso.
- ¿Qué pasa con eventos de todo el día, de varias jornadas o con huso horario
  distinto al del equipo? → se muestran en la franja correcta respetando la zona
  horaria del evento; los recordatorios se calculan sobre instantes absolutos y
  suenan en la hora local correcta.
- ¿Qué pasa con cambios de horario de verano (DST) entre la programación y el
  disparo de un recordatorio? → el recordatorio se calcula sobre instante absoluto
  en UTC, nunca sobre "falta X hh:mm" fija en pared.
- ¿Qué pasa si hay decenas de eventos en un día? → la lista del widget se recorta a
  lo visible con un indicador "+N más" y scroll interno en el panel expandido.
- ¿Qué pasa si un recordatorio cuya hora ya pasó se encuentra al arrancar? → no se
  dispara en cascada; se muestra como "perdido" en la actividad reciente como máximo.
- ¿Qué pasa si dos feeds aportan el mismo evento (invitación repetida)? → se deduplica
  por identidad de evento, mostrando una única entrada con los calendarios
  de origen.
- ¿Qué pasa si el usuario borra su configuración o el archivo se corrompe? → la app
  arranca con estado vacío seguro y una nota editorial de primer uso, sin bloquearse.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: El sistema DEBE permitir añadir, renombrar y eliminar calendarios
  identificando cada uno con nombre visible y URL ICS privada.
- **FR-002**: El sistema DEBE validar que la URL sea HTTPS y devolver un error claro
  si el contenido descargado no es un calendario ICS válido.
- **FR-003**: El sistema DEBE interpretar los eventos del feed incluyendo los
  campos: título, inicio/fin, todo-el-día, ubicación, descripción y organizador.
- **FR-004**: El sistema DEBE expandir eventos recurrentes (RRULE) para los próximos
  7 días visibles, respetando huso horario del evento y del equipo.
- **FR-005**: El sistema DEBE mantener una caché local persistente de la última
  lectura válida de cada feed y usarla ante fallo de red.
- **FR-006**: El sistema DEBE refrescar los feeds automáticamente cada N minutos
  (configurable, por defecto 15) y permitir refresco manual inmediato.
- **FR-007**: El sistema DEBE programar recordatorios nativos de Windows por evento
  según un margen global configurable (por defecto 10 minutos antes).
- **FR-008**: El sistema DEBE deduplicar recordatorios por identidad de evento +
  hora programada, incluso tras refrescos o reinicios.
- **FR-009**: El sistema DEBE cancelar los recordatorios de eventos eliminados o
  reprogramados en el siguiente refresco.
- **FR-010**: El sistema DEBE mostrar el widget como ventana flotante sin marco,
  siempre visible, arrastrable y con click-through en sus zonas transparentes.
- **FR-011**: El widget DEBE mostrar al menos: fecha actual, lista de eventos de hoy
  y próximos días, y estado de conexión de los calendarios.
- **FR-012**: El widget DEBE ofrecer modo contraído (chip) y expandido (agenda
  semanal + detalle), con transiciones fluidas y respetando `prefers-reduced-motion`.
- **FR-013**: El sistema DEBE aplicar el lenguaje de diseño mono·color definido en la
  constitución (principio I): dos tintas con roles, sustrato de papel, tipografía
  Fraunces/Space Grotesk/IBM Plex Mono y texturas dosificadas.
- **FR-014**: El sistema DEBE ofrecer personalización de: tamaño (presets),
  opacidad, sustrato (white/gray/beige), posición, atajo global de mostrar/ocultar,
  intervalo de refresco, margen de recordatorio y autoarranque.
- **FR-015**: El sistema DEBE persistir toda la configuración y cachés en el
  directorio de configuración local estándar, sin enviar datos fuera del equipo
  (principio IV).
- **FR-016**: El sistema DEBE detectar URLs ICS idénticas añadidas dos veces e
  impedir calendarios duplicados.
- **FR-017**: El panel de ajustes DEBE mostrar el estado de salud de cada calendario
  (ok/error/sin conexión) con indicadores no dependientes del color únicamente.
- **FR-018**: El sistema DEBE ofrecer un estado vacío editorial (sin eventos hoy)
  que siga el lenguaje de diseño y ofrezca la acción de añadir un calendario.

### Key Entities *(include if feature involves data)*

- **CalendarFeed**: un calendario conectado; nombre visible, URL ICS, estado de
  salud, última lectura exitosa, activo/inactivo.
- **CalendarEvent**: un evento expandido; identidad única (uid del feed + recurrencia
  + inicio), título, inicio/fin instantes absolutos, todo-el-día, ubicación,
  descripción, origen (feed), series de recurrencia.
- **ReminderPlan**: un recordatorio programado; referencia al evento, hora absoluta
  de disparo (UTC), estado (programado/disparado/cancelado/perdido), firma de
  deduplicación.
- **WidgetSettings**: tamaño, opacidad, sustrato, posición, contraído/expandido,
  atajo global, intervalo de refresco, margen de recordatorio, autoarranque.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Un usuario conecta su primer calendario y ve sus eventos en ≤ 2 minutos
  desde que abre los ajustes por primera vez.
- **SC-002**: El 100 % de los recordatorios programados se disparan dentro de ±5 s
  de su hora objetivo en una sesión normal, medido en pruebas automatizadas del
  planificador.
- **SC-003**: En reposo, el widget registra 0 % de CPU perceptible y ≤ 80 MB de
  memoria en el Administrador de Tareas de Windows.
- **SC-004**: El arranque en frío muestra el widget respondiendo en ≤ 1.5 s en
  hardware de gama media.
- **SC-005**: Con conexión cortada, el widget conserva el último estado válido, no
  pierde recordatorios ya planificados con datos en caché y muestra claramente el
  estado "sin conexión".
- **SC-006**: El binario resultante ocupa ≤ 15 MB y no requiere más de 80 MB de
  memoria en ningún estado interactivo normal.
- **SC-007**: Las 5 historias de usuario son demostrables de forma independiente en
  una sesión de revisión, sin pasos de preparación ocultos.

## Assumptions

- Usuario único en un único equipo Windows; la sincronización multi-dispositivo está
  fuera del alcance de la v1.
- Acceso a Google Calendar solo lectura vía URL ICS secreta; crear/editar/borrar
  eventos desde el widget queda fuera del alcance de la v1 (requeriría OAuth).
- El usuario de Google Calendar sabe obtener la "dirección secreta en iCal" (los
  ajustes del propio widget lo explicarán con el microcopy adecuado).
- Los feeds ICS de Google no incluyen los recordatorios (VALARM) configurados en
  Google, por lo que los márgenes de recordatorio se definen en el widget.
- Windows 10/11 con WebView2 disponible; se incluye en el instalador si falta.
- Sin dependencias de servidores intermedios: tráfico HTTPS directo del equipo del
  usuario a los endpoints de Google.

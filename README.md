<br />

<p align="center">
  <img src="docs/app-icon.png" width="104" alt="mono·calendar" />
</p>

<h1 align="center">mono·calendar</h1>

<p align="center">
  <em>La agenda del día, impresa sobre tu escritorio.</em><br/>
  Widget de calendario para Windows que lee tu feed privado de Google Calendar
  (iCal, solo lectura) y anuncia cada cita <strong>antes de que ocurra</strong> —
  silencioso, liviano y con estética editorial en dos tintas.
</p>

<p align="center">
  <a href="./LICENSE"><img alt="Licencia MIT" src="https://img.shields.io/badge/licencia-MIT-2148B8?style=flat-square" /></a>
  <img alt="Windows 10/11" src="https://img.shields.io/badge/Windows-10%20%2F%2011-0078D6?style=flat-square&logo=windows&logoColor=white" />
  <img alt="Tauri 2" src="https://img.shields.io/badge/Tauri-2-FFC131?style=flat-square" />
  <img alt="Rust" src="https://img.shields.io/badge/Rust-stable-DEA584?style=flat-square&logo=rust" />
  <img alt="React 18" src="https://img.shields.io/badge/React-18-61DAFB?style=flat-square&logo=react&logoColor=black" />
  <img alt="TypeScript" src="https://img.shields.io/badge/TypeScript-5-3178C6?style=flat-square&logo=typescript&logoColor=white" />
  <img alt="tests" src="https://img.shields.io/badge/tests-23%20%2F%2023%20verde-46A971?style=flat-square" />
</p>

---

<p align="center">
  <img src="docs/widget-chip.png" width="380" alt="Chip contraído: 27 · martes · sin eventos" />&nbsp;&nbsp;
  <img src="docs/widget-agenda.png" width="286" alt="Agenda expandida: Papel en blanco, estado editorial" /><br/>
  <sub>El chip contraído vive sobre tu escritorio · la agenda expandida abre con estado editorial</sub>
</p>

## ¿Por qué mono·calendar?

Los calendarios de escritorio suelen ser aplicaciones pesadas que exigen
ventana, cuenta y atención. **mono·calendar** es lo contrario: un chip
flotante del tamaño de una línea de titular que vive sobre tu escritorio,
muestra *solo lo que falta por ocurrir* y te avisa con una notificación
nativa de Windows a tiempo — sin instalar cuentas, sin sincronizar nada más
que tu feed iCal privado, sin salir de tu equipo.

- **Solo lectura y 100 % local.** Conecta la «dirección secreta en iCal» de tu
  Google Calendar; el widget descarga, interpreta y Agenda. Nada sale de tu
  máquina más allá de esa descarga.
- **Recordatorios que sobreviven a todo.** El planificador persiste cada aviso
  en disco antes de dispararlo: si el widget se reinicia, descubre lo pendiente
  y lo entrega o lo marca como perdido. Nunca duplica, nunca olvida en silencio.
- **Presupuesto de recursos medidos** (build release, Windows 11): RAM **34,6 MB**
  en reposo, arranque **633 ms** hasta ventana visible, CPU ≈ **0 %** en reposo,
  binario **10,7 MB**, instalador NSIS **3,1 MiB** — todos los gates de la
  constitución del proyecto, verificados.
- **Una estética con nombre propio.** Papel, dos tintas (cobalto dominante,
  terracota como acento), grano e imprenta: la interfaz podría ser una página
  de revista sobre tu monitor.

## Características

| | |
|---|---|
| 🗓 **Agenda viva** | Chip contraído con fecha grande + próximo evento; al expandir, HOY y próximos 8 días agrupados por día con recorte «+N más». |
| ⏰ **Recordatorios fiables** | Aviso configurable (10 min por defecto) + aviso temprano opcional; dedupe por firma `{identidad}@{margen}`; cancelación reactiva si el evento se mueve, se elimina o cambia el margen (≤ 1 s). |
| ♻️ **Recurrencia real** | Parser y expansor ICS propios sobre el subconjunto RFC 5545 que exporta Google: `DAILY`, `WEEKLY+BYDAY`, `MONTHLY`, `INTERVAL`, `UNTIL`, `COUNT`, `EXDATE`, `RECURRENCE-ID` y TZID (DST correcto por instante absoluto UTC). |
| 🎛 **Personalización en vivo** | Tres sustratos de papel, tres tamaños, opacidad, margen de aviso y frecuencia de refresco; cada cambio se aplica al instante y persiste. |
| 🩺 **Salud visible** | Cada feed reporta `sincronizado / error / sin conexión`; la barra de estado combina texto + marca no cromática (nunca solo color), y los datos en caché sobreviven a la caída de red. |
| 🖱 **Comportamiento de escritorio** | Arrastre libre con posición memorizada, colapso/expansión animada, click-through real en zonas transparentes, atajo global `Ctrl+Alt+C` (editable) y autoarranque con Windows. |
| 🌐 **Formato local** | Interfaz y mensajes en español; hora local en pantalla, hora UTC por dentro (DST-proof). |

## Filosofía de diseño — mono·color

mono·calendar sigue el sistema **mono·color**: un solo sustrato de papel,
como máximo dos tintas con roles fijos, texturas dosificadas y
prescindibles, tipografía con tres voces y movimiento sobrio.

<p align="center">
  <img alt="papel #FAFAF7" src="https://img.shields.io/badge/papel-FAFAF7-FAFAF7?style=for-the-badge&labelColor=FAFAF7" />
  <img alt="cobalto #2148B8" src="https://img.shields.io/badge/cobalto-2148B8-2148B8?style=for-the-badge" />
  <img alt="terracota #C65F38" src="https://img.shields.io/badge/terracota-C65F38-C65F38?style=for-the-badge" />
  <img alt="tinta #20242A" src="https://img.shields.io/badge/tinta-20242A-20242A?style=for-the-badge" />
</p>

- **Fraunces itálica** para las fechas monumentales (la voz de portada).
- **Space Grotesk** para el cuerpo de la agenda.
- **IBM Plex Mono** para horas, etiquetas técnicas y contadores.
- Grano de papel y tramas *halftone* con opacidad mínima; al retirarlas la
  interfaz sigue funcionando igual (una regla dura del sistema).
- `prefers-reduced-motion` elimina todo el movimiento.

## Arquitectura

```mermaid
flowchart LR
    subgraph FRONT["Front · React 18 + TS"]
        W["Widget<br/>chip ⇄ agenda"]
        Cfg["Ajustes en vivo"]
    end

    subgraph CORE["Core · Rust (Tauri 2)"]
        F["ICS<br/>fetch · parse · expand"]
        S["Scheduler<br/>tick 1 s · dedupe · replay"]
        N["Notificación<br/>nativa Windows"]
        DB[("Store JSON<br/>%APPDATA%/mono-calendar")]
    end

    G["Google Calendar<br/>(iCal privado, solo lectura)"] -->|HTTPS| F
    F --> S
    S --> N
    S <--> DB
    W <-->|IPC tipado| S
    Cfg <-->|IPC tipado| DB
```

El núcleo decide *cuándo* avisar; el front solo *muestra*. Los recordatorios
se calculan sobre instantes absolutos UTC (nunca matemática de pared), lo que
hace que los avisos sobrevivan cambios de zona horaria, DST y reinicios.

## Empezar

### Requisitos

- Windows 10 u 11 (WebView2 Runtime, incluido en Windows 11)
- [Rust stable (MSVC)](https://rustup.rs)
- Node.js ≥ 20
- Visual Studio Build Tools 2022 (workload «Desktop development with C++»)

### Desarrollo

```powershell
git clone https://github.com/Javiervalenciam/mono-calendar.git
cd mono-calendar
npm install
npm run tauri dev
```

La primera compilación de Rust tarda varios minutos; las siguientes, segundos.

### Conectar tu Google Calendar (solo lectura)

1. En Google Calendar → **Configuración** → *configuración de mi calendario*.
2. Copia la **«Dirección secreta en iCal»** (URL terminada en `.ics`).
3. En mono·calendar → **Ajustes → 01 Calendarios** pega la URL y dale
   **conectar**. Listo: los eventos aparecen en ≤ 10 s.

> La URL secreta equivale a una contraseña de lectura de ese calendario:
> compártela solo con dispositivos de tu confianza. mono·calendar la guarda
> únicamente en tu carpeta de usuario local y la **ofusca** en cualquier
> mensaje de error o registro.

### Compilar instalador

```powershell
npm run tauri build
# → src-tauri/target/release/bundle/nsis/mono-calendar_0.1.0_x64-setup.exe (≈ 3,1 MiB)
```

## Privacidad por diseño

- **Una sola conexión saliente**: la descarga del feed iCal que tú configuras.
  Sin telemetría, sin analytics, sin pings.
- **Un solo lugar de escritura**: `%APPDATA%/mono-calendar/` (settings, feeds,
  recordatorios y caché por feed, con escritura atómica `tempfile + rename`).
- Errores de red con URL **redactada** (`…/calendarios/<redacted>`), jamás la
  dirección secreta completa.
- Los datos en caché permiten trabajar **sin conexión**: la agenda muestra los
  últimos datos y el estado de salud lo declara honestamente.

## Calidad y pruebas

La lógica crítica tiene suite automatizada (23 tests sobre 7 fixtures ICS):

```powershell
cd src-tauri
cargo test
```

- **Parser/expansión**: eventos simples, all-day, `DAILY/COUNT`, `WEEKLY+BYDAY`,
  `MONTHLY`, TZID Bogotá, cruce DST Madrid, `EXDATE`.
- **Scheduler**: creación y conservación de planes, no-duplicación por firma,
  cancelación por evento eliminado, reprogramación por evento movido,
  `missed` al arrancar con plan vencido, gracia de 5 minutos, horizonte 24 h,
  dedupe entre feeds (uid + inicio + título).
- **Storage**: ida y vuelta, sanitizado de rangos, autocura de archivos
  corruptos y tope de 20 entradas en el registro reciente.

## Estructura del proyecto

```text
mono-calendar/
├── src/                    # Front React 18 + TypeScript + Tailwind v4
│   ├── components/         # Widget, AgendaList, EventDetail, SettingsPanel, StatusBar
│   ├── components/editorial/  # RegMark + elipse editorial (gestos de imprenta)
│   ├── lib/                # ipc.ts (puente tipado), time.ts, theme.ts
│   └── styles/             # tokens mono·color, grano, halftone, movimiento
├── src-tauri/              # Core Rust (Tauri 2)
│   ├── src/ics/            # fetch (If-Modified-Since) · parse · expand (RFC 5545)
│   ├── src/scheduler.rs    # plan/dedupe/cancelación/replay — persistencia-first
│   ├── src/store.rs        # %APPDATA% — escritura atómica y autocura
│   ├── src/notify.rs       # toasts nativos (única fuente de avisos)
│   └── tests/              # 23 tests + 7 fixtures ICS
└── specs/                  # Especificación Spec Kit del proyecto
    └── 001-calendar-widget/   # constitución, spec, plan, contratos, quickstart
```

`specs/` documenta el proceso completo con la metodología Spec Kit: la
constitución del proyecto (recursos, fiabilidad, privacidad, diseño y
accesibilidad como gates verificables), los contratos IPC de almacenamiento y
el plan de investigación con sus desviaciones registradas.

## Roadmap

- [ ] Icono en bandeja con menú (mostrar/ocultar, salir)
- [ ] Recordar posición por monitor (multi-pantalla)
- [ ] Vista semana y búsqueda rápida
- [ ] Internacionalización (EN)
- [ ] Acciones en la notificación («empezar ahora», «aplazar»)
- [ ] CI con GitHub Actions (tests en cada push)

## Autor

**Javier Valencia M.** — Colombia 🇨🇴

[GitHub · @Javiervalenciam](https://github.com/Javiervalenciam)

## Licencia

El código se publica bajo [licencia MIT](./LICENSE). Las tipografías y
bibliotecas de terceros conservan sus licencias originales — ver
[THIRD-PARTY-NOTICES.md](./THIRD-PARTY-NOTICES.md).

Los datos de calendario que leas con esta herramienta siguen siendo de su
propietario: mono·calendar no los replica fuera de tu equipo.

<!--
Agradecimientos silenciosos: este proyecto se construyó con la ayuda de
modelos de IA gratuitos y el IDE OpenCode 2.0 — bajo la dirección, las
decisiones de producto y la revisión de Javier Valencia M.
-->

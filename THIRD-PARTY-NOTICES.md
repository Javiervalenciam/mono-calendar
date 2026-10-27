# Avisos de terceros — mono·calendar

mono·calendar es software propio bajo licencia [MIT](./LICENSE). Este aviso
resume las licencias de los componentes y recursos de terceros que se
distribuyen con la aplicación.

## Tipografías (autoalojadas vía @fontsource)

| Fuente | Titular | Licencia |
|---|---|---|
| Fraunces | The Fraunces Project Authors | SIL Open Font License 1.1 |
| Space Grotesk | Florian Karsten | SIL Open Font License 1.1 |
| IBM Plex Mono | IBM Corp. | SIL Open Font License 1.1 |

Las fuentes OFL se distribuyen como archivos de fuente y pueden usarse,
estudiarlas, modificarse y redistribuirse libremente; no se aplican al código
que las consume. Fuente completa disponible en
<https://openfontlicense.org>.

## Íconos y bibliotecas

| Componente | Licencia |
|---|---|
| [lucide-react](https://lucide.dev) | ISC |
| React, React DOM | MIT |
| Vite | MIT |
| Tailwind CSS v4 | MIT |
| TypeScript | Apache-2.0 |
| Tauri 2 (core y plugins) | MIT / Apache-2.0 |

Los crates de Rust del archivo `src-tauri/Cargo.lock` conservan sus propias
licencias (mayoritariamente MIT u MIT/Apache-2.0). Consulte la sección
`[package]` de cada crate para el detalle legal exacto.

## Datos del usuario

Los calendarios leídos mediante URL iCal privada de Google Calendar siguen
siendo propiedad del usuario y de Google. mono·calendar solo los lee,
almacena una copia de trabajo cifrada por el propio equipo en
`%APPDATA%/mono-calendar/` y nunca envía esos datos a servidores distintos
del feed configurado.

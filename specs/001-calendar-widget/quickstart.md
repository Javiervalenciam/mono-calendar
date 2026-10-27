# Quickstart: validación end-to-end de mono·calendar

Fecha: 2026-10-27. Escenarios ejecutables que demuestran que la feature funciona.
Referencias: [plan.md](./plan.md) · [data-model.md](./data-model.md) ·
[contracts/ipc-commands.md](./contracts/ipc-commands.md)

## Prerrequisitos (una sola vez)

1. VS Build Tools 2022 con workload C++ (MSVC v143 + Windows SDK) — instalado
   durante esta sesión vía winget si faltaba.
2. Rust stable-msvc (`rustup`), `cargo` en PATH.
3. Node + npm (ya presentes). WebView2 Runtime (ya presente).
4. Primera compilación del core: `npm install && npm run tauri dev`
   (la primera compilación de Rust tarda varios minutos; las siguientes, segundos).

## Validación de lógica (automática)

```powershell
cd src-tauri
cargo test
```

Debe pasar el suite completo, incluyendo:

- Fixture ICS con evento simple, todo-el-día, recurrente semanal (RRULE:FREQ=WEEKLY),
  recurrente mensual y TZID distinto al del sistema → expansión exacta de los
  próximos 8 días.
- Caso DST: evento programado cruzando el cambio horario → instante UTC correcto.
- Scheduler: plan → dedupe (misma firma no se reprograma) → evento eliminado ⇒
  `cancelled` → arranque con recordatorio vencido ⇒ `missed` sin disparo.
- Storage: escritura atómica y arranque con archivos corruptos ⇒ defaults + aviso.

## Validación de UI (manual, ≤ 15 min)

Ejecutar `npm run tauri dev` y recorrer las historias de usuario:

1. **US-1 Ver eventos**: primer arranque muestra estado vacío editorial con CTA
   "Añadir calendario". Pegar una URL ICS secreta real de Google Calendar →
   eventos de hoy/próximos 7 días visibles en ≤ 10 s. Clic en un evento → detalle.
2. **US-2 Recordatorios**: crear en Google un evento a ~12 min. En ajustes fijar
   margen 10 min. Esperar → toast nativo de Windows con título y "comienza en
   10 min". Refrescar de nuevo → SIN notificación duplicada. Eliminar el evento
   en Google y refrescar → al llegar la hora NO debe sonar nada.
3. **US-2 offline**: cortar Wi-Fi tras tener eventos en caché; crear evento en
   un feed de prueba estático no es posible — alternativa: verificar que el plan
   de recordatorios sigue activo (`reminders.json` intacto) y que al llegar la
   hora sin red se dispara (la notificación no depende de red).
4. **US-3 Personalizar**: cambiar sustrato (white/gray/beige), tamaño, opacidad
   y margen → cambios en vivo; cerrar y reabrir → persistidos.
5. **US-4 Salud**: añadir un feed con URL modificada/inválida → estado "error"
   visible (texto + indicador, no solo color), sin borrar su config. Sin Wi-Fi →
   "sin conexión — usando últimos datos".
6. **US-5 Escritorio**: arrastrar el widget (persiste posición), clic para
   expandir/contraer, Ctrl+Alt+C para ocultar/mostrar, clic en zona transparente
   atraviesa, habilitar autoarranque y reiniciar sesión.

## Gates de cierre ( checklist de constitución )

- [x] **Rendimiento (II)** — medido en build release (2026-10-27): RAM **34,6 MB**
      tras arranque (+8 s; gate ≤ 80 MB), arranque a ventana visible **633 ms**
      (gate ≤ 1,5 s), CPU total 0,17 s incluyendo el arranque (≈ 0 % en reposo),
      binario **10,74 MB** (gate ≤ 15 MB) e instalador NSIS **3,09 MiB**.
- [x] **Diseño (I)** — auditoría §22 en [checklists/design-gate.md](./checklists/design-gate.md):
      un sustrato, dos tintas con rol, texturas removibles — **PASS**.
- [x] **Fiabilidad (III)** — suite `cargo test` **23/23 verde**; no-duplicación
      probada por firma (doble compute ⇒ 0 planes nuevos) en scheduler_test.
- [x] **Privacidad (IV)** — única salida de red = feed configurado; única
      escritura = `%APPDATA%/mono-calendar/`; URLs redactadas en errores
      (verificable en `src-tauri/src/ics/fetch.rs` y `src-tauri/src/store.rs`).
- [ ] **Accesibilidad (VI)** — `prefers-reduced-motion` y foco visible
      implementados en código; pendiente verificación manual con Tab y OS
      en modo de movimiento reducido.
- [ ] **US-2/E2E manual** — toast real con evento de Google a ~12 min;
      sin duplicados tras refrescos; cancelación al eliminar el evento.

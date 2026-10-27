// Mapa de sustratos y presets de tamaño — lib/theme (T017).
import type { Size, Substrate } from "./ipc";

export const SUBSTRATE_CSS: Record<Substrate, string> = {
  white: "#FAFAF7",
  gray: "#E9E9E5",
  beige: "#F5F1E8",
};

export const SUBSTRATE_LABEL: Record<Substrate, string> = {
  white: "Papel blanco",
  gray: "Gris frío",
  beige: "Beige pálido",
};

export const SIZE_PRESETS: Record<Size, { width: number; collapsed: number; expanded: number }> = {
  s: { width: 320, collapsed: 104, expanded: 520 },
  m: { width: 360, collapsed: 112, expanded: 560 },
  l: { width: 460, collapsed: 124, expanded: 620 },
};

export const SIZE_LABEL: Record<Size, string> = {
  s: "Compacto",
  m: "Mediano",
  l: "Grande",
};

/** Aplica el sustrato como variable CSS global (T031, en vivo). */
export function applySubstrate(s: Substrate): void {
  document.documentElement.style.setProperty("--substrate", SUBSTRATE_CSS[s]);
}

/** Opacidad del contenido (US-3.1) — CSS, porque Tauri 2 no la expone nativa. */
export function applyOpacity(o: number): void {
  document.documentElement.style.setProperty("--app-opacity", String(o));
}

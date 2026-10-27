// Formateo local es-ES — lib/time (T017).
const hhmmFmt = new Intl.DateTimeFormat("es-ES", { hour: "2-digit", minute: "2-digit" });

export function hhmm(iso: string): string {
  return hhmmFmt.format(new Date(iso));
}

export function dayKey(d: Date): string {
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(
    d.getDate(),
  ).padStart(2, "0")}`;
}

export function isToday(dateStr: string): boolean {
  return dayKey(new Date()) === dateStr;
}

const WD = ["DOMINGO", "LUNES", "MARTES", "MIÉRCOLES", "JUEVES", "VIERNES", "SÁBADO"];
const MO = ["ENE", "FEB", "MAR", "ABR", "MAY", "JUN", "JUL", "AGO", "SEP", "OCT", "NOV", "DIC"];

/** Etiqueta de sección: HOY / MAÑANA / "MARTES 5 · NOV". */
export function dayLabel(dateStr: string): string {
  if (isToday(dateStr)) return "HOY";
  const tomorrow = new Date(Date.now() + 86_400_000);
  if (dayKey(tomorrow) === dateStr) return "MAÑANA";
  const d = new Date(dateStr + "T12:00:00");
  return `${WD[d.getDay()]} ${d.getDate()} · ${MO[d.getMonth()]}`;
}

export function bigDate(d: Date): { day: string; month: string; weekday: string } {
  return {
    day: String(d.getDate()),
    month: `${MO[d.getMonth()]} ${d.getFullYear()}`,
    weekday: WD[d.getDay()],
  };
}

export function minutesUntil(iso: string): number {
  return Math.round((Date.parse(iso) - Date.now()) / 60_000);
}

/** Estado humano de un evento relativo a ahora. */
export function eventState(ev: { start_utc: string; end_utc: string; all_day: boolean }): string {
  if (ev.all_day) return "todo el día";
  const m = minutesUntil(ev.start_utc);
  const endM = Math.round((Date.parse(ev.end_utc) - Date.now()) / 60_000);
  if (m <= 0 && endM > 0) return "en curso";
  if (endM <= 0) return "terminó";
  if (m <= 60) return `en ${m} min`;
  return hhmm(ev.start_utc);
}

export function hhmmShort(iso: string): string {
  const d = new Date(iso);
  return String(d.getMinutes()).padStart(2, "0");
}

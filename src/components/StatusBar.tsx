// Barra de estado — salud de feeds (US-4): nunca solo color, texto + marca.
import type { CalendarFeed } from "../lib/ipc";
import { hhmm } from "../lib/time";
import { RegMark } from "./editorial/RegMark";

interface Props {
  feeds: CalendarFeed[];
  anyStale: boolean;
}

export function StatusBar({ feeds, anyStale }: Props) {
  const active = feeds.filter((f) => f.active);
  const errors = active.filter((f) => f.health === "error");
  const offlines = active.filter((f) => f.health === "offline");
  const lastOk = active
    .map((f) => f.last_fetch_ok_at)
    .filter((v): v is string => !!v)
    .sort()
    .at(-1) ?? null;

  let text = "SINCRONIZADO";
  let tone = "text-cobalt";
  let marker = "■"; // marca no-dependiente-del-color

  if (errors.length > 0) {
    text = errors.length === 1 ? "ERROR EN 1 CALENDARIO" : `ERROR EN ${errors.length} CALENDARIOS`;
    tone = "text-terra";
  } else if (offlines.length > 0 || anyStale) {
    text = lastOk ? `SIN CONEXIÓN · ÚLTIMOS DATOS ${hhmm(lastOk)}` : "SIN DATOS · SIN CONEXIÓN";
    tone = "text-lead";
  } else if (active.length === 0) {
    text = "SIN CALENDARIOS";
  }

  if (errors.length > 0) marker = "▲";
  else if (offlines.length > 0 || anyStale) marker = "○";

  return (
    <div className="mt-2 flex items-center gap-2 border-t border-ink/10 px-5 py-2">
      <RegMark className="shrink-0 text-lead opacity-60" />
      <span className={`font-mono text-[9px] uppercase tracking-[0.16em] ${tone}`}>
        <span className="mr-1.5">{marker}</span>
        {text}
      </span>
      <span className="h-px flex-1 bg-ink/10" />
      <span className="font-mono text-[9px] tracking-[0.14em] text-lead">
        {active.length} CAL
      </span>
    </div>
  );
}

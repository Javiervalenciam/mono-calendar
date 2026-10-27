// AgendaList — hoy + próximos días, recorte "+N más", estados editoriales.
import type { AgendaDay, CalendarEvent } from "../lib/ipc";
import { dayLabel, eventState, hhmm } from "../lib/time";
import { EditorialEllipse, RegMark } from "./editorial/RegMark";

const VISIBLE_PER_DAY = 4;

interface Props {
  days: AgendaDay[] | null;
  hasFeeds: boolean;
  anyStale: boolean;
  onSelect: (ev: CalendarEvent) => void;
  onAddFeed: () => void;
}

export function AgendaList({ days, hasFeeds, anyStale, onSelect, onAddFeed }: Props) {
  if (!days) {
    return (
      <div className="animate-reveal px-5 py-10 text-center">
        <p className="font-mono text-[10px] uppercase tracking-[0.2em] text-lead">
          Cargando…</p>
      </div>
    );
  }

  // Primer arranque sin calendarios: estado vacío editorial (FR-018).
  if (!hasFeeds) {
    return (
      <div className="animate-reveal relative flex-1 overflow-hidden px-6 py-8 text-center">
        <div className="halftone pointer-events-none absolute inset-0" />
        <div className="relative">
          <RegMark className="mx-auto mb-4 text-cobalt" />
          <h2 className="font-display text-[26px] italic leading-tight text-ink">
            Papel en blanco
          </h2>
          <p className="mx-auto mt-2 max-w-[24ch] text-[12.5px] leading-relaxed text-lead">
            Añade tu calendario de Google y verás aquí tus próximos días.
          </p>
          <button
            onClick={onAddFeed}
            className="mt-5 font-mono text-[11px] uppercase tracking-[0.18em] text-terra underline decoration-terra/40 decoration-2 underline-offset-4 transition-colors hover:decoration-terra"
          >
            Añadir calendario →
          </button>
        </div>
      </div>
    );
  }

  const flat = days.flatMap((d) => d.events);

  if (flat.length === 0) {
    return (
      <div className="animate-reveal relative flex-1 overflow-hidden px-6 py-10 text-center">
        <div className="halftone pointer-events-none absolute inset-0" />
        <div className="relative">
          <h2 className="font-display text-[24px] italic text-ink">
            <span className="relative inline-block">
              Sin eventos
              <EditorialEllipse className="absolute -inset-x-3 -inset-y-1 h-[calc(100%+8px)] w-[calc(100%+24px)] animate-floaty" />
            </span>
          </h2>
          <p className="mt-3 font-mono text-[10px] uppercase tracking-[0.2em] text-lead">
            Próximos 8 días en blanco
          </p>
          <p className="mt-4 text-[12px] text-lead">
            Silencio en la página: buen momento para el trabajo largo.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="animate-reveal flex-1 overflow-y-auto px-5 pb-3">
      {anyStale && (
        <p className="mb-2 mt-1 border border-terra/30 bg-terra/[0.05] px-2.5 py-1.5 font-mono text-[9px] uppercase tracking-[0.16em] text-terra">
          ○ sin conexión — mostrando los últimos datos guardados
        </p>
      )}
      {days.map((day) => {
        if (day.events.length === 0) return null;
        return (
          <section key={day.date} className="mt-3 first:mt-1">
            <h3 className="mb-1 flex items-baseline gap-2">
              <span className="font-mono text-[10px] uppercase tracking-[0.2em] text-cobalt">
                {dayLabel(day.date)}
              </span>
              <span className="h-px flex-1 bg-ink/10" />
              <span className="font-mono text-[9px] tracking-[0.14em] text-lead">
                {day.events.length}
              </span>
            </h3>
            <ul>
              {day.events.slice(0, VISIBLE_PER_DAY).map((ev) => (
                <li key={ev.identity}>
                  <button
                    onClick={() => onSelect(ev)}
                    className="group flex w-full items-baseline gap-3 border-b border-ink/[0.06] py-2 text-left transition-colors hover:bg-ink/[0.03] focus-visible:bg-ink/[0.03]"
                  >
                    <span className="w-[52px] shrink-0 font-mono text-[11px] tracking-[0.08em] text-cobalt">
                      {ev.all_day ? "DÍA" : hhmm(ev.start_utc)}
                    </span>
                    <span className="min-w-0 flex-1 truncate text-[13px] leading-snug text-ink">
                      {ev.title}
                    </span>
                    <span className="shrink-0 font-mono text-[9px] uppercase tracking-[0.12em] text-lead">
                      {eventState(ev)}
                    </span>
                  </button>
                </li>
              ))}
              {day.events.length > VISIBLE_PER_DAY && (
                <li className="py-1.5 font-mono text-[10px] tracking-[0.14em] text-terra">
                  +{day.events.length - VISIBLE_PER_DAY} MÁS EN ESTE DÍA
                </li>
              )}
            </ul>
          </section>
        );
      })}
    </div>
  );
}

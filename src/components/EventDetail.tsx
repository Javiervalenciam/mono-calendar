// Detalle de evento — panel superpuesto con animación reveal ≤250 ms (T024).
import type { CalendarEvent } from "../lib/ipc";
import { dayKey, dayLabel, hhmm } from "../lib/time";

interface Props {
  event: CalendarEvent;
  feedName: string | null;
  onBack: () => void;
}

export function EventDetail({ event, feedName, onBack }: Props) {
  const start = new Date(event.start_utc);
  const end = new Date(event.end_utc);
  const sameDay = dayKey(start) === dayKey(end) || event.all_day;
  const range = event.all_day
    ? dayLabel(dayKey(start))
    : sameDay
      ? `${hhmm(event.start_utc)} – ${hhmm(event.end_utc)}`
      : `${hhmm(event.start_utc)} → ${hhmm(event.end_utc)}`;

  return (
    <div className="animate-reveal flex-1 overflow-y-auto px-6 pb-5">
      <button
        onClick={onBack}
        className="mb-3 font-mono text-[10px] uppercase tracking-[0.2em] text-cobalt transition-colors hover:text-cobalt-deep"
      >
        ← Volver
      </button>

      <h2 className="font-display text-[22px] italic leading-tight text-ink">
        {event.title}
      </h2>
      <p className="mt-1 font-mono text-[11px] tracking-[0.14em] text-cobalt">
        {range}
      </p>

      <dl className="mt-4 space-y-3 border-t border-ink/10 pt-4 text-[12.5px]">
        {event.location && (
          <div>
            <dt className="font-mono text-[9px] uppercase tracking-[0.2em] text-lead">
              Ubicación
            </dt>
            <dd className="mt-0.5 text-ink">{event.location}</dd>
          </div>
        )}
        {event.description && (
          <div>
            <dt className="font-mono text-[9px] uppercase tracking-[0.2em] text-lead">
              Notas
            </dt>
            <dd className="mt-0.5 whitespace-pre-line leading-relaxed text-ink">
              {event.description}
            </dd>
          </div>
        )}
        <div className="flex items-center gap-2">
          <dt className="font-mono text-[9px] uppercase tracking-[0.2em] text-lead">
            Origen
          </dt>
          <dd className="font-mono text-[10.5px] tracking-[0.08em] text-lead">
            {feedName ?? event.feed_id}
          </dd>
        </div>
      </dl>

      {event.is_recurring && (
        <p className="mt-4 inline-flex items-center gap-1.5 bg-ink/[0.04] px-2 py-1 font-mono text-[9px] uppercase tracking-[0.16em] text-lead">
          <span className="text-terra">↻</span> serie recurrente
        </p>
      )}
    </div>
  );
}

// Widget — chip contraído ⇄ agenda expandida (US-1, US-5.2, FR-010/011/012).
import { ChevronDown, ChevronUp, RefreshCw, Settings as SettingsIcon, X } from "lucide-react";
import { useState, type ReactNode } from "react";
import type { Agenda, CalendarEvent, CalendarFeed } from "../lib/ipc";
import { bigDate, hhmm } from "../lib/time";
import { AgendaList } from "./AgendaList";
import { EventDetail } from "./EventDetail";
import { StatusBar } from "./StatusBar";

interface Props {
  agenda: Agenda | null;
  feeds: CalendarFeed[];
  expanded: boolean;
  onToggle: () => void;
  onOpenSettings: () => void;
  onQuit: () => void;
  onRefresh: () => void;
  onDrag: () => void;
}

function IconBtn({
  onClick,
  label,
  children,
  className = "",
}: {
  onClick: () => void;
  label: string;
  children: ReactNode;
  className?: string;
}) {
  return (
    <button
      onClick={onClick}
      title={label}
      aria-label={label}
      onMouseDown={(e) => e.stopPropagation()}
      className={`rounded-[2px] p-1.5 text-lead transition-colors hover:bg-ink/[0.05] hover:text-ink ${className}`}
    >
      {children}
    </button>
  );
}

export function Widget({
  agenda,
  feeds,
  expanded,
  onToggle,
  onOpenSettings,
  onQuit,
  onRefresh,
  onDrag,
}: Props) {
  const [selected, setSelected] = useState<CalendarEvent | null>(null);
  const today = new Date();
  const { day, month, weekday } = bigDate(today);

  const todayEvents = agenda?.days[0]?.events ?? [];
  const remaining = todayEvents.filter((e) => Date.parse(e.end_utc) > Date.now());
  const nextEv = remaining[0] ?? null;
  const soon =
    nextEv !== null &&
    Date.parse(nextEv.start_utc) - Date.now() <= 30 * 60_000 &&
    Date.parse(nextEv.start_utc) > Date.now();

  const feedName = (id: string) => feeds.find((f) => f.id === id)?.name ?? null;

  if (!expanded) {
    return (
      <div
        onMouseDown={onDrag}
        title="Arrástrame · ⌄ abre la agenda"
        className="pointer-events-auto flex h-full cursor-grab select-none items-center border border-ink/10 shadow-[0_16px_38px_-18px_rgba(32,36,42,0.38)] active:cursor-grabbing"
        style={{ background: "var(--substrate)", opacity: "var(--app-opacity, 1)" }}
      >
        <div className="flex w-full items-center gap-4 px-5 py-3">
          <div className="flex items-baseline gap-2.5">
            <span className="font-display text-[40px] italic leading-none text-cobalt">
              {day}
            </span>
            <div className="flex flex-col gap-0.5">
              <span className="font-mono text-[8.5px] tracking-[0.24em] text-lead">
                {month}
              </span>
              <span className="font-mono text-[10px] tracking-[0.2em] text-ink">
                {weekday}
              </span>
            </div>
          </div>

          <div className="h-10 w-px shrink-0 bg-ink/10" />

          <div className="min-w-0 flex-1">
            {nextEv ? (
              <>
                <p className="flex items-center gap-1.5 font-mono text-[9px] uppercase tracking-[0.18em] text-lead">
                  <span className={soon ? "text-terra" : "text-cobalt"}>●</span>
                  {remaining.length} {remaining.length === 1 ? "EVENTO HOY" : "EVENTOS HOY"}
                </p>
                <p className="mt-0.5 truncate text-[12.5px] leading-snug text-ink">
                  <span className="font-mono text-[10.5px] tracking-[0.06em] text-cobalt">
                    {nextEv.all_day ? "DÍA" : hhmm(nextEv.start_utc)}
                  </span>{" "}
                  {nextEv.title}
                </p>
              </>
            ) : (
              <p className="font-mono text-[9.5px] uppercase leading-relaxed tracking-[0.18em] text-lead">
                día libre — sin eventos
              </p>
            )}
          </div>

          <IconBtn onClick={onToggle} label="Expandir agenda">
            <ChevronDown className="h-4 w-4" />
          </IconBtn>
        </div>
      </div>
    );
  }

  return (
    <div
      className="pointer-events-auto flex h-full flex-col border border-ink/10 shadow-[0_16px_38px_-18px_rgba(32,36,42,0.38)]"
      style={{ background: "var(--substrate)", opacity: "var(--app-opacity, 1)" }}
    >
      <header
        onMouseDown={onDrag}
        className="cursor-grab select-none px-5 pb-2 pt-3.5 active:cursor-grabbing"
      >
        <div className="flex items-start justify-between">
          <div>
            <p className="font-mono text-[9px] uppercase tracking-[0.26em] text-cobalt">
              mono·calendar
            </p>
            <h1 className="font-display text-[28px] italic leading-tight text-ink">
              {weekday.charAt(0) + weekday.slice(1).toLowerCase()}
              <span className="not-italic text-lead">,{" "}</span>
              <span className="text-cobalt">{day}</span>
            </h1>
            <p className="font-mono text-[9px] tracking-[0.22em] text-lead">{month}</p>
          </div>
          <div className="flex items-center gap-0.5" onMouseDown={(e) => e.stopPropagation()}>
            <IconBtn onClick={onRefresh} label="Refrescar ahora">
              <RefreshCw className="h-3.5 w-3.5" />
            </IconBtn>
            <IconBtn onClick={onOpenSettings} label="Ajustes">
              <SettingsIcon className="h-3.5 w-3.5" />
            </IconBtn>
            <IconBtn onClick={onToggle} label="Contraer">
              <ChevronUp className="h-4 w-4" />
            </IconBtn>
            <IconBtn onClick={onQuit} label="Salir">
              <X className="h-3.5 w-3.5" />
            </IconBtn>
          </div>
        </div>
        <div className="h-px w-full bg-ink/10" />
      </header>

      <main className="flex min-h-0 flex-1 flex-col">
        {selected ? (
          <EventDetail
            event={selected}
            feedName={feedName(selected.feed_id)}
            onBack={() => setSelected(null)}
          />
        ) : (
          <AgendaList
            days={agenda?.days ?? null}
            hasFeeds={feeds.length > 0}
            anyStale={agenda?.any_stale ?? false}
            onSelect={setSelected}
            onAddFeed={onOpenSettings}
          />
        )}
      </main>

      <StatusBar feeds={feeds} anyStale={agenda?.any_stale ?? false} />
    </div>
  );
}

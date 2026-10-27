// Panel de ajustes — secciones 01–04, cambios en vivo y persistentes
// (US-3, US-4; T030–T032, T034).
import { Plus, RefreshCw, Trash2, X } from "lucide-react";
import { useState, type ReactNode } from "react";
import {
  api,
  type CalendarFeed,
  type Size,
  type Substrate,
  type WidgetSettings,
} from "../lib/ipc";
import { SIZE_LABEL, SIZE_PRESETS, SUBSTRATE_LABEL, applySubstrate } from "../lib/theme";
import { RegMark } from "./editorial/RegMark";

interface Props {
  settings: WidgetSettings;
  feeds: CalendarFeed[];
  onSaved: (s: WidgetSettings) => void;
  onReload: () => void;
  onClose: () => void;
}

function Section({
  index,
  title,
  children,
}: {
  index: string;
  title: string;
  children: ReactNode;
}) {
  return (
    <section className="mt-6">
      <h3 className="flex items-baseline gap-2">
        <span className="font-mono text-[10px] tracking-[0.14em] text-terra">{index}</span>
        <span className="font-mono text-[11px] uppercase tracking-[0.22em] text-ink">
          {title}
        </span>
        <span className="h-px flex-1 bg-ink/10" />
      </h3>
      <div className="mt-3 space-y-3">{children}</div>
    </section>
  );
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="block">
      <span className="mb-1 block font-mono text-[9px] uppercase tracking-[0.2em] text-lead">
        {label}
      </span>
      {children}
    </label>
  );
}

const inputCls =
  "w-full border border-ink/15 bg-transparent px-2.5 py-1.5 text-[12.5px] text-ink placeholder:text-lead/60 focus:border-cobalt focus:outline-none";

function Segmented<T extends string>({
  options,
  value,
  onChange,
}: {
  options: { v: T; label: string }[];
  value: T;
  onChange: (v: T) => void;
}) {
  return (
    <div className="flex border border-ink/15">
      {options.map((o, i) => (
        <button
          key={o.v}
          onClick={() => onChange(o.v)}
          aria-pressed={value === o.v}
          className={`px-3 py-1.5 font-mono text-[10px] uppercase tracking-[0.14em] transition-colors ${
            i > 0 ? "border-l border-ink/15" : ""
          } ${
            value === o.v
              ? "bg-cobalt text-paper"
              : "text-lead hover:bg-ink/[0.04] hover:text-ink"
          }`}
        >
          {o.label}
        </button>
      ))}
    </div>
  );
}

function Check({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <label className="flex cursor-pointer items-center gap-2 text-[12.5px] text-ink">
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        className="h-3.5 w-3.5 accent-cobalt"
      />
      {label}
    </label>
  );
}

const HEALTH_TEXT: Record<string, string> = {
  ok: "sincronizado",
  error: "error",
  offline: "sin conexión",
};
const HEALTH_MARK: Record<string, string> = { ok: "■", error: "▲", offline: "○" };
const HEALTH_CLS: Record<string, string> = {
  ok: "text-cobalt",
  error: "text-terra",
  offline: "text-lead",
};

export function SettingsPanel({ settings, feeds, onSaved, onReload, onClose }: Props) {
  const [name, setName] = useState("");
  const [url, setUrl] = useState("");
  const [feedError, setFeedError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [editing, setEditing] = useState<string | null>(null);
  const [editName, setEditName] = useState("");
  const [deleting, setDeleting] = useState<string | null>(null);

  const save = async (patch: Partial<WidgetSettings>) => {
    const saved = await api.saveSettings({ ...settings, ...patch });
    applySubstrate(saved.substrate);
    onSaved(saved);
  };

  const addFeed = async () => {
    setFeedError(null);
    setBusy(true);
    try {
      await api.addFeed(name.trim(), url.trim());
      setName("");
      setUrl("");
      onReload();
    } catch (e) {
      setFeedError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const patchFeed = async (f: CalendarFeed) => {
    await api.updateFeed(f);
    onReload();
  };

  return (
    <div
      className="pointer-events-auto flex h-full flex-col border border-ink/10 shadow-[0_16px_38px_-18px_rgba(32,36,42,0.38)]"
      style={{ background: "var(--substrate)", opacity: "var(--app-opacity, 1)" }}
    >
      <header className="flex items-center justify-between border-b border-ink/10 px-5 py-3">
        <div className="flex items-center gap-2.5">
          <RegMark className="text-cobalt" />
          <h2 className="font-display text-[20px] italic text-ink">Ajustes</h2>
        </div>
        <button
          onClick={onClose}
          aria-label="Cerrar ajustes"
          className="p-1.5 text-lead transition-colors hover:bg-ink/[0.05] hover:text-ink"
        >
          <X className="h-4 w-4" />
        </button>
      </header>

      <div className="flex-1 overflow-y-auto px-5 pb-6">
        {/* ── 01 CALENDARIOS ─────────────────────────────────── */}
        <Section index="01" title="Calendarios">
          {feeds.map((f) => (
            <div key={f.id} className="border-b border-ink/[0.07] pb-2.5">
              <div className="flex items-center gap-2">
                {editing === f.id ? (
                  <input
                    value={editName}
                    autoFocus
                    onChange={(e) => setEditName(e.target.value)}
                    onKeyDown={async (e) => {
                      if (e.key === "Enter" && editName.trim()) {
                        await patchFeed({ ...f, name: editName.trim() });
                        setEditing(null);
                      }
                      if (e.key === "Escape") setEditing(null);
                    }}
                    className={inputCls}
                  />
                ) : (
                  <button
                    onClick={() => patchFeed({ ...f, active: !f.active })}
                    title={f.active ? "Desactivar calendario" : "Activar calendario"}
                    className="flex min-w-0 flex-1 items-baseline gap-2 text-left"
                  >
                    <span className={`font-mono text-[10px] ${HEALTH_CLS[f.health]}`}>
                      {HEALTH_MARK[f.health]}
                    </span>
                    <span
                      className={`truncate text-[13px] ${
                        f.active ? "text-ink" : "text-lead line-through"
                      }`}
                    >
                      {f.name}
                    </span>
                  </button>
                )}
                <span
                  className={`shrink-0 font-mono text-[9px] uppercase tracking-[0.14em] ${HEALTH_CLS[f.health]}`}
                >
                  {HEALTH_TEXT[f.health]}
                </span>
                {deleting === f.id ? (
                  <>
                    <button
                      onClick={async () => {
                        await api.removeFeed(f.id);
                        setDeleting(null);
                        onReload();
                      }}
                      className="font-mono text-[9.5px] uppercase tracking-[0.12em] text-terra"
                    >
                      sí
                    </button>
                    <button
                      onClick={() => setDeleting(null)}
                      className="font-mono text-[9.5px] uppercase tracking-[0.12em] text-lead"
                    >
                      no
                    </button>
                  </>
                ) : (
                  <button
                    onClick={() => setDeleting(f.id)}
                    aria-label={`Eliminar ${f.name}`}
                    className="p-1 text-lead transition-colors hover:text-terra"
                  >
                    <Trash2 className="h-3.5 w-3.5" />
                  </button>
                )}
              </div>
              {editing === f.id ? (
                <p className="mt-1 font-mono text-[9px] tracking-[0.1em] text-lead">
                  ENTER guarda · ESC cancela
                </p>
              ) : (
                <button
                  onClick={() => {
                    setEditing(f.id);
                    setEditName(f.name);
                  }}
                  className="mt-0.5 block w-full truncate text-left font-mono text-[9px] tracking-[0.08em] text-lead/70 hover:text-lead"
                  title="Renombrar"
                >
                  {f.health === "error" ? f.last_error ?? "" : "renombrar · tocar para editar"}
                </button>
              )}
            </div>
          ))}

          <div className="space-y-2 border border-ink/10 p-3">
            <div className="flex gap-2">
              <input
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="Nombre (ej. Personal)"
                className={inputCls}
              />
              <input
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                placeholder="URL iCal secreta (https://…)"
                className={`${inputCls} flex-1 font-mono text-[11px]`}
              />
            </div>
            {feedError && (
              <p className="font-mono text-[10px] leading-relaxed text-terra">{feedError}</p>
            )}
            <button
              onClick={addFeed}
              disabled={busy || !name.trim() || !url.trim()}
              className="inline-flex items-center gap-1.5 border border-cobalt px-3 py-1.5 font-mono text-[10px] uppercase tracking-[0.16em] text-cobalt transition-colors hover:bg-cobalt hover:text-paper disabled:cursor-not-auto disabled:opacity-40"
            >
              <Plus className="h-3.5 w-3.5" /> conectar
            </button>
            <p className="font-mono text-[9px] leading-relaxed tracking-[0.06em] text-lead/80">
              En Google Calendar: ajustes del calendario → «Dirección secreta en
              iCal» → copia esa URL aqui. Solo lectura; nada sale de tu equipo.
            </p>
          </div>

          <button
            onClick={async () => {
              await api.refreshFeeds();
              onReload();
            }}
            className="inline-flex items-center gap-1.5 font-mono text-[10px] uppercase tracking-[0.16em] text-cobalt hover:text-cobalt-deep"
          >
            <RefreshCw className="h-3 w-3" /> refrescar ahora
          </button>
        </Section>

        {/* ── 02 APARIENCIA ──────────────────────────────────── */}
        <Section index="02" title="Apariencia">
          <Field label="Sustrato de papel">
            <Segmented<Substrate>
              value={settings.substrate}
              onChange={(v) => save({ substrate: v })}
              options={(Object.keys(SUBSTRATE_LABEL) as Substrate[]).map((k) => ({
                v: k,
                label: SUBSTRATE_LABEL[k],
              }))}
            />
          </Field>
          <Field label="Tamaño">
            <Segmented<Size>
              value={settings.size}
              onChange={(v) => save({ size: v })}
              options={(Object.keys(SIZE_PRESETS) as Size[]).map((k) => ({
                v: k,
                label: SIZE_LABEL[k],
              }))}
            />
          </Field>
          <Field label={`Opacidad — ${Math.round(settings.opacity * 100)}%`}>
            <input
              type="range"
              min={0.5}
              max={1}
              step={0.05}
              value={settings.opacity}
              onChange={(e) => save({ opacity: Number(e.target.value) })}
              className="w-full accent-cobalt"
            />
          </Field>
        </Section>

        {/* ── 03 RECORDATORIOS ───────────────────────────────── */}
        <Section index="03" title="Recordatorios">
          <div className="flex gap-3">
            <Field label="Avisar (min antes)">
              <input
                type="number"
                min={0}
                max={120}
                value={settings.reminder_offset_minutes}
                onChange={(e) =>
                  save({ reminder_offset_minutes: Math.max(0, Math.round(Number(e.target.value) || 0)) })
                }
                className={`${inputCls} w-[88px]`}
              />
            </Field>
            <Field label="Aviso temprano (opcional)">
              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={settings.urgent_offset_minutes !== null}
                  onChange={(e) =>
                    save({ urgent_offset_minutes: e.target.checked ? 60 : null })
                  }
                  className="h-3.5 w-3.5 accent-cobalt"
                />
                <input
                  type="number"
                  min={0}
                  max={120}
                  disabled={settings.urgent_offset_minutes === null}
                  value={settings.urgent_offset_minutes ?? 60}
                  onChange={(e) =>
                    save({
                      urgent_offset_minutes: Math.max(0, Math.round(Number(e.target.value) || 0)),
                    })
                  }
                  className={`${inputCls} w-[88px] disabled:opacity-40`}
                />
              </div>
            </Field>
          </div>
          <Check
            label="Sonido de notificación"
            checked={settings.sound}
            onChange={(v) => save({ sound: v })}
          />
          <button
            onClick={() => api.testNotification()}
            className="font-mono text-[10px] uppercase tracking-[0.16em] text-terra underline decoration-terra/40 underline-offset-4 hover:decoration-terra"
          >
            notificación de prueba →
          </button>
        </Section>

        {/* ── 04 SISTEMA ─────────────────────────────────────── */}
        <Section index="04" title="Sistema">
          <div className="flex gap-3">
            <Field label="Refresco (minutos)">
              <input
                type="number"
                min={5}
                max={240}
                value={settings.refresh_minutes}
                onChange={(e) =>
                  save({ refresh_minutes: Math.round(Number(e.target.value) || 15) })
                }
                className={`${inputCls} w-[88px]`}
              />
            </Field>
            <Field label="Atajo mostrar/ocultar">
              <input
                value={settings.global_shortcut}
                onChange={(e) => save({ global_shortcut: e.target.value })}
                className={`${inputCls} w-[140px] font-mono text-[11px]`}
              />
            </Field>
          </div>
          <Check
            label="Arrancar con Windows"
            checked={settings.autostart}
            onChange={(v) => save({ autostart: v })}
          />
          <button
            onClick={() => api.quitApp()}
            className="font-mono text-[10px] uppercase tracking-[0.16em] text-lead hover:text-terra"
          >
            cerrar el widget por completo →
          </button>
        </Section>
      </div>
    </div>
  );
}

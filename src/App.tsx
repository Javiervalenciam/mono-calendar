// App — orquesta ventana, estado IPC y comportamiento de escritorio
// (US-5.1/5.2/5.3; contracts/ipc-commands.md).
import { useCallback, useEffect, useRef, useState } from "react";
import {
  getCurrentWindow,
  LogicalPosition,
  LogicalSize,
} from "@tauri-apps/api/window";
import {
  api,
  onAgendaUpdated,
  onFeedsUpdated,
  type Agenda,
  type CalendarFeed,
  type WidgetSettings,
} from "./lib/ipc";
import { SIZE_PRESETS, applyOpacity, applySubstrate } from "./lib/theme";
import { Widget } from "./components/Widget";
import { SettingsPanel } from "./components/SettingsPanel";

export default function App() {
  const [settings, setSettings] = useState<WidgetSettings | null>(null);
  const [agenda, setAgenda] = useState<Agenda | null>(null);
  const [feeds, setFeeds] = useState<CalendarFeed[]>([]);
  const [view, setView] = useState<"widget" | "settings">("widget");
  const [expanded, setExpanded] = useState(false);
  const settingsRef = useRef<WidgetSettings | null>(null);
  settingsRef.current = settings;

  // Carga inicial + suscripciones push del core.
  useEffect(() => {
    let alive = true;
    (async () => {
      const [s, a, f] = await Promise.all([
        api.getSettings(),
        api.getAgenda(),
        api.listFeeds(),
      ]);
      if (!alive) return;
      setSettings(s);
      setExpanded(!s.collapsed);
      applySubstrate(s.substrate);
      applyOpacity(s.opacity);
      if (s.position.x >= 0 && s.position.y >= 0) {
        getCurrentWindow().setPosition(new LogicalPosition(s.position.x, s.position.y));
      }
      setAgenda(a);
      setFeeds(f);
    })();
    const u1 = onAgendaUpdated((a) => setAgenda(a));
    const u2 = onFeedsUpdated(() => {
      api.listFeeds().then((f) => {
        if (alive) setFeeds(f);
      });
    });
    return () => {
      alive = false;
      u1.then((fn) => fn());
      u2.then((fn) => fn());
    };
  }, []);

  // Tamaño de la ventana según preset + estado expand/contraída (US-3.1/5.2).
  useEffect(() => {
    if (!settings) return;
    const p = SIZE_PRESETS[settings.size];
    const h = expanded ? p.expanded : p.collapsed;
    getCurrentWindow().setSize(new LogicalSize(p.width, h));
  }, [settings?.size, expanded]);

  // Sustrato y opacidad EN VIVO (US-3.1) — CSS, Tauri 2 no expone set-opacity.
  useEffect(() => {
    if (!settings) return;
    applySubstrate(settings.substrate);
    applyOpacity(settings.opacity);
  }, [settings?.substrate, settings?.opacity]);

  // Persistir posición tras arrastre, con debounce (US-5.1).
  useEffect(() => {
    if (!settings) return;
    let timer: number | undefined;
    const un = getCurrentWindow().onMoved(async () => {
      window.clearTimeout(timer);
      timer = window.setTimeout(async () => {
        const cur = settingsRef.current;
        if (!cur) return;
        try {
          const w = getCurrentWindow();
          const pos = await w.outerPosition();
          const sf = await w.scaleFactor();
          const l = pos.toLogical(sf);
          const x = Math.round(l.x);
          const y = Math.round(l.y);
          if (x !== cur.position.x || y !== cur.position.y) {
            const saved = await api.saveSettings({ ...cur, position: { x, y } });
            setSettings(saved);
          }
        } catch {
          // La ventana puede haberse cerrado: nada que hacer.
        }
      }, 500);
    });
    return () => {
      un.then((fn) => fn());
      window.clearTimeout(timer);
    };
  }, [settings]);

  const patchSettings = useCallback(
    async (patch: Partial<WidgetSettings>) => {
      if (!settings) return;
      const saved = await api.saveSettings({ ...settings, ...patch });
      setSettings(saved);
    },
    [settings],
  );

  const refresh = useCallback(async () => {
    const f = await api.refreshFeeds();
    setFeeds(f);
  }, []);

  if (!settings) {
    return (
      <div
        className="pointer-events-auto flex h-full items-center justify-center"
        style={{ background: "var(--substrate)" }}
      >
        <p className="font-mono text-[10px] tracking-[0.32em] text-lead">
          MONO·CALENDAR
        </p>
      </div>
    );
  }

  return (
    // Zona transparente: pointer-events none ⇒ el clic atraviesa (US-5.3).
    <div className="h-full" style={{ pointerEvents: "none" }}>
      {view === "settings" ? (
        <SettingsPanel
          settings={settings}
          feeds={feeds}
          onSaved={setSettings}
          onReload={refresh}
          onClose={() => {
            setView("widget");
            if (settings.collapsed) setExpanded(false);
          }}
        />
      ) : (
        <Widget
          agenda={agenda}
          feeds={feeds}
          expanded={expanded}
          onToggle={() => {
            const next = !expanded;
            setExpanded(next);
            patchSettings({ collapsed: !next });
          }}
          onOpenSettings={() => {
            setView("settings");
            setExpanded(true);
          }}
          onQuit={() => api.quitApp()}
          onRefresh={refresh}
          onDrag={() => getCurrentWindow().startDragging()}
        />
      )}
    </div>
  );
}

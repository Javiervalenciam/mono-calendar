// Puente IPC tipado — contracts/ipc-commands.md.
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type Health = "ok" | "error" | "offline";
export type Accent = "cobalt" | "terra";
export type Substrate = "white" | "gray" | "beige";
export type Size = "s" | "m" | "l";

export interface CalendarFeed {
  id: string;
  name: string;
  url: string;
  active: boolean;
  health: Health;
  last_error: string | null;
  last_fetch_ok_at: string | null;
  last_modified: string | null;
  accent: Accent;
}

export interface CalendarEvent {
  identity: string;
  feed_id: string;
  uid: string;
  title: string;
  start_utc: string;
  end_utc: string;
  all_day: boolean;
  location: string | null;
  description: string | null;
  is_recurring: boolean;
  series_id: string | null;
}

export interface AgendaDay {
  date: string; // YYYY-MM-DD local
  events: CalendarEvent[];
}

export interface Agenda {
  days: AgendaDay[];
  generated_at: string;
  any_stale: boolean;
}

export interface WidgetSettings {
  version: number;
  substrate: Substrate;
  size: Size;
  opacity: number;
  position: { x: number; y: number };
  collapsed: boolean;
  refresh_minutes: number;
  reminder_offset_minutes: number;
  urgent_offset_minutes: number | null;
  global_shortcut: string;
  autostart: boolean;
  sound: boolean;
}

export const api = {
  getSettings: () => invoke<WidgetSettings>("get_settings"),
  saveSettings: (settings: WidgetSettings) =>
    invoke<WidgetSettings>("save_settings", { settings }),
  listFeeds: () => invoke<CalendarFeed[]>("list_feeds"),
  addFeed: (name: string, url: string) =>
    invoke<CalendarFeed>("add_feed", { name, url }),
  updateFeed: (feed: CalendarFeed) =>
    invoke<CalendarFeed>("update_feed", { feed }),
  removeFeed: (feedId: string) => invoke<void>("remove_feed", { feedId }),
  refreshFeeds: () => invoke<CalendarFeed[]>("refresh_feeds"),
  getAgenda: () => invoke<Agenda>("get_agenda"),
  testNotification: () => invoke<void>("test_notification"),
  quitApp: () => invoke<void>("quit_app"),
};

export function onAgendaUpdated(cb: (a: Agenda) => void): Promise<UnlistenFn> {
  return listen<Agenda>("agenda-updated", (e) => cb(e.payload));
}

export function onFeedsUpdated(cb: () => void): Promise<UnlistenFn> {
  return listen("feeds-updated", () => cb());
}

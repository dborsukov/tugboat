/**
 * Update state shared by the main window and the About window.
 *
 * Each Tauri webview has its own JS context, so the main window owns the
 * updater state and broadcasts snapshots over these events. The About window
 * only mirrors them and asks the main window to act.
 */

export const UPDATE_STATUS_EVENT = "update-status";
export const UPDATE_STATUS_REQUEST = "update-status-request";
export const UPDATE_CHECK_REQUEST = "update-check-request";
export const UPDATE_SHOW_REQUEST = "update-show-request";

export type UpdateStatus =
  | { state: "idle" }
  | { state: "checking" }
  | { state: "up-to-date"; checkedAt: number }
  | { state: "available"; version: string; checkedAt: number }
  | { state: "error"; checkedAt: number };

/** `HH:MM` in the local time zone, for the About status lines. */
export function formatCheckTime(value: number): string {
  return new Intl.DateTimeFormat("uk-UA", {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(new Date(value));
}

import type { HandleClientError } from "@sveltejs/kit";
import { error as logError } from "@tauri-apps/plugin-log";

import { dev } from "$app/environment";

const MAX_DETAIL_LENGTH = 4000;

/** Crashes already reported this session, keyed by their first line. */
const seen = new Map<string, number>();

/** First line of multiline string, used to populate `seen` */
function signatureOf(text: string): string {
  return text.split("\n", 1)[0];
}

function repeatSuffix(signature: string): string | null {
  const count = (seen.get(signature) ?? 0) + 1;
  seen.set(signature, count);
  if (count > 3 && count % 10 !== 0) return null;
  return count === 1 ? "" : ` (x${count})`;
}

function truncate(text: string): string {
  if (text.length <= MAX_DETAIL_LENGTH) return text;
  return `${text.slice(0, MAX_DETAIL_LENGTH)}… [+${text.length - MAX_DETAIL_LENGTH} chars]`;
}

function describe(error: unknown): string {
  if (error instanceof Error) {
    return truncate(error.stack ?? `${error.name}: ${error.message}`);
  }
  if (typeof error === "string") return truncate(error);
  try {
    return truncate(JSON.stringify(error) ?? String(error));
  } catch {
    return String(error);
  }
}

/** Logging should never crash the app, check if logger is available */
function log(line: string) {
  if (dev) console.error(line);
  try {
    void logError(line).catch(() => {});
  } catch {
    // No logger available, console only
  }
}

export const handleError: HandleClientError = ({ error, event, status }) => {
  const detail = describe(error);
  const repetition = repeatSuffix(signatureOf(detail));
  if (repetition !== null) {
    const route = event.route.id ? ` (${event.route.id})` : "";
    log(
      `[Svelte Rendering Error]${repetition} [${status}] ${event.url.pathname}${route}\n${detail}`,
    );
  }

  return {
    message:
      "Не вдалося завантажити сторінку. Спробуйте оновити вікно (Ctrl+R).",
  };
};

if (typeof window !== "undefined") {
  // Unhandled promise
  window.addEventListener("unhandledrejection", (event) => {
    const detail = describe(event.reason);
    const repetition = repeatSuffix(signatureOf(detail));
    if (repetition !== null) {
      log(`[Unhandled Rejection]${repetition} at ${location.href}\n${detail}`);
    }
  });

  // Generic error
  window.addEventListener("error", (event) => {
    if (event.error) {
      const detail = describe(event.error);
      const repetition = repeatSuffix(signatureOf(detail));
      if (repetition !== null) {
        log(`[Uncaught Error]${repetition} at ${location.href}\n${detail}`);
      }
      return;
    }

    const source = event.filename
      ? `${event.filename}:${event.lineno}:${event.colno}`
      : "unknown source";
    // Keyed by the source
    const repetition = repeatSuffix(`${event.message} at ${source}`);
    if (repetition !== null) {
      log(`[Uncaught Error]${repetition} ${event.message} at ${source}`);
    }
  });
}

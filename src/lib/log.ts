import { error as logError, info as logInfo } from "@tauri-apps/plugin-log";

import { dev } from "$app/environment";

/**
 * Client-side logging.
 *
 * The frontend is the only layer that knows which command was rejected, so
 * command failures are logged here (see the `MutationCache` / `QueryCache`
 * hooks in `+layout.svelte`). Successful business operations are logged by the
 * Rust side as audit events, where the domain data lives.
 *
 * Everything goes through `@tauri-apps/plugin-log`, so it lands in the same
 * `app.log` as the backend lines, tagged `webview`.
 */

/** `AppError` arrives as a plain `{ type, message }` object, not an `Error`. */
type AppErrorShape = { type?: string; message?: string };

const MAX_DETAIL_LENGTH = 4000;

const seen = new Map<string, number>();

function describe(error: unknown): { kind: string; detail: string } {
  if (
    error &&
    typeof error === "object" &&
    "type" in error &&
    "message" in error
  ) {
    const appError = error as AppErrorShape;
    return {
      kind: appError.type ?? "unknown",
      detail: String(appError.message ?? ""),
    };
  }
  if (error instanceof Error) {
    return {
      kind: "exception",
      detail: error.stack ?? `${error.name}: ${error.message}`,
    };
  }
  try {
    return { kind: "value", detail: JSON.stringify(error) ?? String(error) };
  } catch {
    return { kind: "value", detail: String(error) };
  }
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

/** Logging must never break the app, so every transport failure is swallowed. */
function emit(level: "error" | "info", line: string) {
  if (dev) {
    if (level === "error") console.error(line);
    else console.info(line);
  }
  try {
    const send = level === "error" ? logError : logInfo;
    void send(line).catch(() => {});
  } catch {
    // No logger available
  }
}

/**
 * A command rejected by the backend. Written as one line.
 * `operation` is the mutation key (e.g. `clients.createClient`) or the query key
 */
export function logCommandFailure(
  operation: string,
  error: unknown,
  context?: string,
) {
  const { kind, detail } = describe(error);
  const firstLine = detail.split("\n", 1)[0];
  const suffix = repeatSuffix(`${operation}:${kind}:${firstLine}`);
  if (suffix === null) return;

  const where = context ? ` ${context}` : "";
  emit(
    "error",
    `[command failed] ${operation}${where}${suffix} kind=${kind} msg=${JSON.stringify(truncate(detail))}`,
  );
}

export function logAppEvent(
  action: string,
  fields: Record<string, string | number | boolean> = {},
) {
  const pairs = Object.entries(fields)
    .map(([key, value]) => {
      const text = String(value);
      const needsQuoting = text === "" || /[\s"=]/.test(text);
      return `${key}=${needsQuoting ? JSON.stringify(text) : text}`;
    })
    .join(" ");

  emit("info", `[app] ${action}${pairs ? ` ${pairs}` : ""}`);
}

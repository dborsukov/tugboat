import type { HandleClientError } from "@sveltejs/kit";
import { error as logError } from "@tauri-apps/plugin-log";

function handleCrash(msg: string) {
  logError(`[Unhandled Critical Error]: ${msg}`);
}

export const handleError: HandleClientError = ({ error }) => {
  const msg = error instanceof Error ? error.message : String(error);
  logError(`[Svelte Rendering Error]: ${msg}`);
  return { message: "Помилка бібліотеки рендерінгу." };
};

if (typeof window !== "undefined") {
  window.addEventListener("unhandledrejection", (event) => {
    const msg =
      event.reason instanceof Error
        ? event.reason.message
        : String(event.reason?.message || event.reason);
    handleCrash(msg);
  });
  window.addEventListener("error", (event) => {
    handleCrash(`${event.message} at ${event.filename}:${event.lineno}`);
  });
}

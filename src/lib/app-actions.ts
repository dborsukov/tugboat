import { toast } from "svelte-sonner";

import { commands } from "$lib/bindings";

const POSITION = "bottom-left" as const;

/** Opens the log directory, surfacing failures as a toast. */
export async function openLogs() {
  const result = await commands.openLogsDir();
  if (result.status === "error") {
    toast.error("Не вдалося відкрити теку журналів", {
      description: result.error.message,
      position: POSITION,
    });
  }
}

/** Copies the diagnostics report, confirming it with a toast. */
export async function copyDiagnostics() {
  const result = await commands.copyDiagnostics();
  if (result.status === "error") {
    toast.error("Не вдалося скопіювати діагностику", {
      description: result.error.message,
      position: POSITION,
    });
    return;
  }
  toast.success("Діагностику скопійовано", { position: POSITION });
}

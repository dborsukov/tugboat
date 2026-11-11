import { emit } from "@tauri-apps/api/event";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { check } from "@tauri-apps/plugin-updater";
import { toast } from "svelte-sonner";

import { dev } from "$app/environment";

import { logAppEvent } from "$lib/log";
import { updateState } from "$lib/states/update.svelte";

/**
 * In-app updates.
 *
 * The updater plugin reads `latest.json` from the newest GitHub release and
 * verifies every download against the public key in `tauri.conf.json`.
 */

const POSITION = "bottom-left" as const;

function isRunningInTauri() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function describeFailure(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

export async function checkForUpdates({ silent = true } = {}) {
  await updateState.check({ silent });
}

export async function requestUpdateCheck() {
  if (dev || !isRunningInTauri()) {
    toast.info("Оновлення доступні лише у зібраному застосунку", {
      position: POSITION,
    });
    return;
  }

  let found;
  try {
    found = await check();
  } catch (error) {
    logAppEvent("update.check_failed", { error: describeFailure(error) });
    toast.error("Не вдалося перевірити оновлення", {
      description: describeFailure(error),
      position: POSITION,
    });
    return;
  }

  if (!found) {
    logAppEvent("update.up_to_date");
    toast.success("У вас встановлена найновіша версія", {
      position: POSITION,
    });
    return;
  }

  await found.close();
  await emit("update-requested");

  const main = await WebviewWindow.getByLabel("main");
  await main?.setFocus();
}

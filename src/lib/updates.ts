import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { toast } from "svelte-sonner";

import { dev } from "$app/environment";

import { logAppEvent } from "$lib/log";

/**
 * In-app updates.
 *
 * The updater plugin reads `latest.json` from the newest GitHub release and
 * verifies every download against the public key in `tauri.conf.json`.
 */

const POSITION = "bottom-left" as const;
const OUTCOME_TOAST_ID = "app-update";
const PROGRESS_TOAST_ID = "app-update-progress";

function isRunningInTauri() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function describeFailure(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

export async function checkForUpdates({ silent = true } = {}) {
  if (dev || !isRunningInTauri()) {
    if (!silent) {
      toast.info("Оновлення доступні лише у зібраному застосунку", {
        position: POSITION,
      });
    }
    return;
  }

  let update: Update | null;
  try {
    update = await check();
  } catch (error) {
    // Expected when offline, when the release has no `latest.json` yet, or issues with signing keys.
    logAppEvent("update.check_failed", { error: describeFailure(error) });
    if (!silent) {
      toast.error("Не вдалося перевірити оновлення", {
        description: describeFailure(error),
        position: POSITION,
      });
    }
    return;
  }

  if (!update) {
    logAppEvent("update.up_to_date");
    if (!silent) {
      toast.success("У вас встановлена найновіша версія", {
        position: POSITION,
      });
    }
    return;
  }

  logAppEvent("update.available", { version: update.version });

  toast.info(`Доступне оновлення ${update.version}`, {
    id: OUTCOME_TOAST_ID,
    description: update.body?.trim() || undefined,
    duration: 20_000,
    position: POSITION,
    action: {
      label: "Оновити",
      onClick: () => void installUpdate(update),
    },
  });
}

async function installUpdate(update: Update) {
  let downloaded = 0;
  let total: number | null = null;

  toast.loading("Завантаження оновлення…", {
    id: PROGRESS_TOAST_ID,
    position: POSITION,
  });

  try {
    await update.downloadAndInstall((event) => {
      if (event.event === "Started") {
        total = event.data.contentLength ?? null;
        return;
      }
      if (event.event !== "Progress") return;

      downloaded += event.data.chunkLength;
      const percent =
        total === null ? null : Math.round((downloaded / total) * 100);
      toast.loading(
        percent === null
          ? "Завантаження оновлення…"
          : `Завантаження оновлення… ${percent} %`,
        { id: PROGRESS_TOAST_ID, position: POSITION },
      );
    });
  } catch (error) {
    logAppEvent("update.install_failed", {
      version: update.version,
      error: describeFailure(error),
    });
    toast.error("Не вдалося встановити оновлення", {
      id: PROGRESS_TOAST_ID,
      description: describeFailure(error),
      position: POSITION,
    });
    return;
  }

  logAppEvent("update.installed", { version: update.version });

  // Runs only on Mac and Linux since Windows quits the app automatically during update.
  toast.success("Оновлення встановлено", {
    id: PROGRESS_TOAST_ID,
    description: "Перезапустіть застосунок, щоб застосувати зміни.",
    duration: 20_000,
    position: POSITION,
    action: {
      label: "Перезапустити",
      onClick: () => void relaunch(),
    },
  });
}

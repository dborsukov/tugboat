import { openUrl } from "@tauri-apps/plugin-opener";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { toast } from "svelte-sonner";

import { dev } from "$app/environment";

import { logAppEvent } from "$lib/log";

/**
 * The in-app update flow.
 *
 * Lifecycle: `check` finds a release, `downloadNow` fetches it (reporting
 * progress), `install` swaps it in, and the dialog offers a restart. The
 * plugin has no abort API, so cancelling stops the install step.
 */

const SKIPPED_VERSION_KEY = "update.skipped-version";
const REMIND_KEY = "update.remind";
const REMIND_DELAY_MS = 24 * 60 * 60 * 1000;
const RELEASE_PAGE = "https://github.com/dborsukov/tugboat/releases/tag/v";
const RESTART_SECONDS = 15;

/** Bytes/s is averaged over this window so the estimate does not jitter. */
const SPEED_WINDOW_MS = 4_000;

export type UpdatePhase =
  | "idle"
  | "checking"
  | "available"
  | "downloading"
  | "installing"
  | "installed"
  | "error";

type SpeedSample = { at: number; bytes: number };

function isRunningInTauri() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function describeFailure(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

function readJson<T>(key: string): T | null {
  try {
    const raw = localStorage.getItem(key);
    return raw ? (JSON.parse(raw) as T) : null;
  } catch {
    return null;
  }
}

function writeJson(key: string, value: unknown) {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {}
}

class UpdateStore {
  phase = $state<UpdatePhase>("idle");
  currentVersion = $state("");
  version = $state("");
  date = $state<string | null>(null);
  body = $state<string | null>(null);

  downloaded = $state(0);
  total = $state<number | null>(null);
  /** Bytes per second, averaged over the last few seconds. */
  speed = $state<number | null>(null);
  error = $state<string | null>(null);
  restartIn = $state(0);

  #update: Update | null = null;
  /** Bumped on cancel so an in-flight download stops updating the UI. */
  #generation = 0;
  #samples: SpeedSample[] = [];
  #restartTimer: ReturnType<typeof setInterval> | null = null;
  #remindTimer: ReturnType<typeof setTimeout> | null = null;
  /** Remembers when this window opened, to tell a reminder from a previous run. */
  #sessionStartedAt = Date.now();

  /** Whether the dialog should be visible. */
  get open() {
    return this.phase !== "idle" && this.phase !== "checking";
  }

  get percent(): number | null {
    if (!this.total || this.total <= 0) return null;
    return Math.min(100, Math.round((this.downloaded / this.total) * 100));
  }

  get etaSeconds(): number | null {
    if (!this.speed || this.speed <= 0 || !this.total) return null;
    return Math.max(0, (this.total - this.downloaded) / this.speed);
  }

  async check({ silent = true } = {}) {
    if (dev || !isRunningInTauri()) {
      if (!silent) {
        toast.info("Оновлення доступні лише у зібраному застосунку", {
          position: "bottom-left",
        });
      }
      return;
    }

    // Never interrupt a download or a pending restart with another check.
    if (this.phase !== "idle" && this.phase !== "error") return;

    this.phase = "checking";

    let found: Update | null;
    try {
      found = await check();
    } catch (error) {
      this.phase = "idle";
      logAppEvent("update.check_failed", { error: describeFailure(error) });
      if (!silent) {
        toast.error("Не вдалося перевірити оновлення", {
          description: describeFailure(error),
          position: "bottom-left",
        });
      }
      return;
    }

    if (!found) {
      this.phase = "idle";
      logAppEvent("update.up_to_date");
      if (!silent) {
        toast.success("У вас встановлена найновіша версія", {
          position: "bottom-left",
        });
      }
      return;
    }

    // The silent boot check respects "skip this version" and "remind me later", an explicit check always triggers.
    if (silent && this.#isDismissed(found.version)) {
      void found.close();
      this.phase = "idle";
      logAppEvent("update.dismissed_by_preference", { version: found.version });
      return;
    }

    this.#update = found;
    this.currentVersion = found.currentVersion;
    this.version = found.version;
    this.date = found.date ?? null;
    this.body = found.body?.trim() || null;
    this.error = null;
    this.#resetProgress();
    this.#clearRemindTimer();
    this.phase = "available";
    logAppEvent("update.available", { version: found.version });
  }

  async downloadNow() {
    const update = this.#update;
    if (!update) return;

    const generation = ++this.#generation;
    this.phase = "downloading";
    this.error = null;
    this.#resetProgress();

    try {
      await update.download((event) => {
        if (generation !== this.#generation) return;

        if (event.event === "Started") {
          this.total = event.data.contentLength ?? null;
          this.#samples = [];
          return;
        }
        if (event.event === "Progress") {
          this.downloaded += event.data.chunkLength;
          this.#recordSample();
        }
      });
    } catch (error) {
      if (generation !== this.#generation) return;
      this.error = describeFailure(error);
      this.phase = "error";
      logAppEvent("update.download_failed", {
        version: this.version,
        error: this.error,
      });
      return;
    }

    if (generation !== this.#generation) return;
    this.phase = "installing";

    try {
      await update.install();
    } catch (error) {
      if (generation !== this.#generation) return;
      this.error = describeFailure(error);
      this.phase = "error";
      logAppEvent("update.install_failed", {
        version: this.version,
        error: this.error,
      });
      return;
    }

    logAppEvent("update.installed", { version: this.version });
    this.phase = "installed";
    this.#startRestartCountdown();
  }

  /** Retry after a failed download or install. */
  async retry() {
    if (!this.#update) return;
    await this.downloadNow();
  }

  /**
   * Stops the pending install. The plugin cannot abort an active transfer, so
   * the bytes already requested are discarded when they arrive.
   */
  cancel() {
    if (this.phase !== "downloading" && this.phase !== "error") return;
    this.#generation += 1;
    void this.#update?.close();
    this.#update = null;
    this.phase = "idle";
    this.#resetProgress();
    logAppEvent("update.cancelled", { version: this.version });
  }

  skipVersion() {
    writeJson(SKIPPED_VERSION_KEY, this.version);
    this.#clearRemindTimer();
    logAppEvent("update.skipped", { version: this.version });
    this.close();
  }

  remindLater() {
    const now = Date.now();
    writeJson(REMIND_KEY, {
      version: this.version,
      until: now + REMIND_DELAY_MS,
      at: now,
    });
    logAppEvent("update.reminded", { version: this.version });
    this.#scheduleRemindRecheck();
    this.close();
  }

  /** Closing the dialog without a choice */
  close() {
    void this.#update?.close();
    this.#update = null;
    this.#clearRestartTimer();
    this.phase = "idle";
    this.#resetProgress();
  }

  /**
   * Closes the dialog from its own X or Escape. Returns false while a transfer
   * is running, which keeps the dialog open.
   */
  dismiss(): boolean {
    if (this.phase === "downloading" || this.phase === "installing") {
      return false;
    }
    if (this.phase === "installed") {
      this.#clearRestartTimer();
      this.close();
      return true;
    }
    if (this.phase === "available") {
      this.remindLater();
      return true;
    }
    this.close();
    return true;
  }

  restartNow() {
    this.#clearRestartTimer();
    logAppEvent("update.restarting", { version: this.version });
    void relaunch();
  }

  cancelRestart() {
    this.#clearRestartTimer();
  }

  openReleaseNotes() {
    if (!this.version) return;
    void openUrl(`${RELEASE_PAGE}${this.version}`);
  }

  #isDismissed(version: string): boolean {
    const skipped = readJson<string>(SKIPPED_VERSION_KEY);
    if (skipped === version) return true;

    const remind = readJson<{ version: string; until: number; at?: number }>(
      REMIND_KEY,
    );
    if (!remind || remind.version !== version) return false;

    if ((remind.at ?? 0) < this.#sessionStartedAt) return false;

    return remind.until > Date.now();
  }

  /** Re-checks after the reminder window if the app is still running. */
  #scheduleRemindRecheck() {
    this.#clearRemindTimer();
    this.#remindTimer = setTimeout(() => {
      this.#remindTimer = null;
      void this.check({ silent: true });
    }, REMIND_DELAY_MS);
  }

  #clearRemindTimer() {
    if (this.#remindTimer !== null) {
      clearTimeout(this.#remindTimer);
      this.#remindTimer = null;
    }
  }

  #recordSample() {
    const now = performance.now();
    this.#samples.push({ at: now, bytes: this.downloaded });
    while (
      this.#samples.length > 2 &&
      now - this.#samples[0].at > SPEED_WINDOW_MS
    ) {
      this.#samples.shift();
    }

    const first = this.#samples[0];
    const elapsed = (now - first.at) / 1000;
    if (elapsed > 0.5) {
      this.speed = (this.downloaded - first.bytes) / elapsed;
    }
  }

  #resetProgress() {
    this.downloaded = 0;
    this.total = null;
    this.speed = null;
    this.#samples = [];
  }

  #startRestartCountdown() {
    this.#clearRestartTimer();
    this.restartIn = RESTART_SECONDS;
    this.#restartTimer = setInterval(() => {
      this.restartIn -= 1;
      if (this.restartIn <= 0) {
        this.restartNow();
      }
    }, 1000);
  }

  #clearRestartTimer() {
    if (this.#restartTimer !== null) {
      clearInterval(this.#restartTimer);
      this.#restartTimer = null;
    }
    this.restartIn = 0;
  }
}

export const updateState = new UpdateStore();

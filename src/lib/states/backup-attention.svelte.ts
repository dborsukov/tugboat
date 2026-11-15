import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { commands } from "$lib/bindings";
import type { BackupOverview } from "$lib/bindings";

export const STALE_AFTER_DAYS = 7;
const STALE_AFTER_MS = STALE_AFTER_DAYS * 24 * 60 * 60 * 1000;

/** Stale or failed backups warrant the orange dot, a fresh install does not. */
export function backupNeedsAttention(overview: BackupOverview): boolean {
  if (overview.status.running) return false;
  if (!overview.folderAvailable || overview.status.lastError) return true;

  const latest = overview.backups[0];
  if (!latest) return false;

  const created = new Date(latest.createdAt).getTime();
  return Number.isFinite(created) && Date.now() - created > STALE_AFTER_MS;
}

class BackupAttention {
  needsAttention = $state(false);

  #unlisten: UnlistenFn | null = null;

  async init() {
    if (this.#unlisten) return;
    this.#unlisten = await listen("backup-status", () => {
      void this.refresh();
    });
    await this.refresh();
  }

  destroy() {
    this.#unlisten?.();
    this.#unlisten = null;
  }

  async refresh() {
    const result = await commands.getBackupOverview();
    if (result.status === "error") return;
    this.needsAttention = backupNeedsAttention(result.data);
  }
}

export const backupAttention = new BackupAttention();

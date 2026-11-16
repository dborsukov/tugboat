import { emit, listen, type UnlistenFn } from "@tauri-apps/api/event";
import { toast } from "svelte-sonner";

import { dev } from "$app/environment";

import {
  UPDATE_CHECK_REQUEST,
  UPDATE_SHOW_REQUEST,
  UPDATE_STATUS_EVENT,
  UPDATE_STATUS_REQUEST,
  type UpdateStatus,
} from "$lib/update-status";

/**
 * The About window's mirror of the main window's update state. It never runs the updater itself.
 */
class AboutUpdate {
  status = $state<UpdateStatus>({ state: "idle" });

  #unlisten: UnlistenFn | null = null;

  async init() {
    if (this.#unlisten) return;
    this.#unlisten = await listen<UpdateStatus>(
      UPDATE_STATUS_EVENT,
      (event) => {
        this.status = event.payload;
      },
    );
    await emit(UPDATE_STATUS_REQUEST);
  }

  destroy() {
    this.#unlisten?.();
    this.#unlisten = null;
  }

  check() {
    if (dev) {
      toast.info("Оновлення доступні лише у зібраному застосунку", {
        position: "bottom-left",
      });
      return;
    }
    if (this.status.state === "checking") return;
    // Show the spinner immediately, the main window confirms with a broadcast.
    this.status = { state: "checking" };
    void emit(UPDATE_CHECK_REQUEST);
  }

  show() {
    void emit(UPDATE_SHOW_REQUEST);
  }
}

export const aboutUpdate = new AboutUpdate();

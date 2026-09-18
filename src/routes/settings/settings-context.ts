import { getContext, setContext } from "svelte";

import type { AppSettings } from "$lib/bindings";

const SETTINGS_CONTEXT_KEY = Symbol("settings-context");

// Only the width differs per page now: the bottom padding that keeps the last
// card clear of the window edge lives in `page-layout.svelte` for every page.
export const SETTINGS_CONTENT_WIDTH_CLASS = "max-w-5xl w-full";

export type SettingsContext = {
  draft: AppSettings | null;
};

export function setSettingsContext(value: SettingsContext) {
  setContext(SETTINGS_CONTEXT_KEY, value);
}

export function getSettingsContext(): SettingsContext {
  return getContext(SETTINGS_CONTEXT_KEY);
}

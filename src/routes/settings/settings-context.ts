import { getContext, setContext } from "svelte";

import type { AppSettings } from "$lib/bindings";

const SETTINGS_CONTEXT_KEY = Symbol("settings-context");

export const SETTINGS_CONTENT_WIDTH_CLASS = "max-w-5xl w-full pb-4";

export type SettingsContext = {
  draft: AppSettings | null;
};

export function setSettingsContext(value: SettingsContext) {
  setContext(SETTINGS_CONTEXT_KEY, value);
}

export function getSettingsContext(): SettingsContext {
  return getContext(SETTINGS_CONTEXT_KEY);
}

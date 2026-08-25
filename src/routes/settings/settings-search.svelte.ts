import { compareItems, rankItem } from "@tanstack/match-sorter-utils";

import { goto } from "$app/navigation";
import { page } from "$app/state";

import {
  settingsSearchIndex,
  type SettingsSearchItem,
} from "./settings-search-index";

const FLASH_CLASS = "settings-target-flash";

class SettingsSearch {
  query = $state("");

  results = $derived(this.#match());

  #match(): SettingsSearchItem[] {
    const query = this.query.trim().toLowerCase();
    if (!query) return [];

    return settingsSearchIndex
      .map((item) => ({
        item,
        rank: rankItem(`${item.label} ${item.keywords ?? ""}`, query),
      }))
      .filter(({ rank }) => rank.passed)
      .sort((a, b) => compareItems(a.rank, b.rank))
      .map(({ item }) => item);
  }

  async select(item: SettingsSearchItem) {
    this.query = "";
    if (page.url.pathname !== item.route) {
      await goto(item.route);
    }
    await this.#highlight(item.id);
  }

  async #highlight(id: string) {
    // The page content mounts with a transition after navigation, so wait for
    // the element to appear before scrolling and flashing.
    for (let attempt = 0; attempt < 40; attempt++) {
      const el = document.getElementById(id);
      if (el) {
        el.scrollIntoView({ behavior: "smooth", block: "center" });
        el.classList.add(FLASH_CLASS);
        setTimeout(() => el.classList.remove(FLASH_CLASS), 2000);
        return;
      }
      await new Promise((resolve) => setTimeout(resolve, 50));
    }
  }
}

export const settingsSearch = new SettingsSearch();

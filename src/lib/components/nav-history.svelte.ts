import { browser } from "$app/environment";
import { afterNavigate } from "$app/navigation";

class NavigationHistory {
  #stack = $state<string[]>([]);
  #index = $state(0);
  #initialized = false;

  canGoBack = $derived(this.#index > 0);
  canGoForward = $derived(this.#index < this.#stack.length - 1);

  init() {
    if (this.#initialized || !browser) return;
    this.#initialized = true;

    afterNavigate((nav) => {
      this.record(nav.type, nav.to?.url.href);
    });
  }

  record(type: string, toHref: string | undefined) {
    if (!toHref) return;

    if (this.#stack.length === 0) {
      this.#stack = [toHref];
      this.#index = 0;
      return;
    }

    if (type === "popstate") {
      if (this.#stack[this.#index - 1] === toHref) {
        this.#index -= 1;
      } else if (this.#stack[this.#index + 1] === toHref) {
        this.#index += 1;
      } else {
        this.#stack = [toHref];
        this.#index = 0;
      }
      return;
    }

    this.#stack = [...this.#stack.slice(0, this.#index + 1), toHref];
    this.#index = this.#stack.length - 1;
  }
}

export const navHistory = new NavigationHistory();

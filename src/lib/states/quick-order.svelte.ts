import { page } from "$app/state";

export type QuickOrderContext = {
  carId?: number | null;
  clientId?: number | null;
};

class QuickOrderState {
  open = $state(false);
  context = $state<QuickOrderContext>({});
  /** Bumped on every request so a second open resets a form that is already mounted. */
  requestId = $state(0);

  show(context: QuickOrderContext = {}) {
    this.context = context;
    this.open = true;
    this.requestId += 1;
  }

  close() {
    this.open = false;
  }

  openFromCurrentRoute() {
    const path = page.url.pathname;
    const car = /^\/cars\/(\d+)(?:\/|$)/.exec(path);
    if (car) {
      this.show({ carId: Number(car[1]) });
      return;
    }
    const client = /^\/clients\/(\d+)(?:\/|$)/.exec(path);
    if (client) {
      this.show({ clientId: Number(client[1]) });
      return;
    }
    this.show({});
  }
}

export const quickOrder = new QuickOrderState();

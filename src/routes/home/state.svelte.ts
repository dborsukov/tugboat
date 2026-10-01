import { browser } from "$app/environment";

import { settingsState } from "$lib/states/settings.svelte";

import { analyticsPeriod, calendarMonthOf, type CalendarMonth } from "./period";

/** How often the dashboard updates the clock in ms */
const TICK_MS = 5 * 60 * 1000;

/**
 * Single period every dashboard module reports on, and the month it is set to.
 * Query keys carry the period, so a tick that produces same windows costs no request
 * while pointing the dashboard at another month updates every module with it.
 */
class DashboardPeriodState {
  #now = $state(new Date());
  #picked = $state<CalendarMonth | null>(null);
  #timer: ReturnType<typeof setInterval> | null = null;

  /** Period timezone */
  timeZone = $derived(settingsState.settings?.timezone ?? null);

  /**
   * When the dashboard last read the clock.
   *
   * Public because the period alone does not say how far into the month the shop is:
   * a module drawing money over time needs the day, and it has to come from the same
   * instant every window above was built from.
   */
  get now(): Date {
    return this.#now;
  }

  /** Current timezone aware month */
  currentAnchor = $derived(calendarMonthOf(this.#now, this.timeZone));

  anchor = $derived(this.#picked ?? this.currentAnchor);

  period = $derived(analyticsPeriod(this.#now, this.timeZone, this.anchor));

  /** Points the dashboard at new `month`, or back at the current month when `month` is `null` */
  goToMonth(month: CalendarMonth | null) {
    this.#picked = month;
  }

  /** Starts the period clock */
  start() {
    if (this.#timer != null || !browser) return;
    this.#timer = setInterval(() => {
      this.#now = new Date();
    }, TICK_MS);
  }
}

export const dashboardPeriod = new DashboardPeriodState();

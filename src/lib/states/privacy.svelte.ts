/**
 * Session-only privacy preferences. Never persisted to the database —
 * everything resets when the app restarts.
 */
class PrivacyState {
  /** Hides the "Ціна закупівлі" (baseCost) and "Націнка" (markup) columns in the parts table. */
  hidePricing = $state(false);
  /** Hides the work order notes block. */
  hideNotes = $state(false);

  /** True when every privacy toggle is on. */
  allHidden = $derived(this.hidePricing && this.hideNotes);

  setAll(hidden: boolean) {
    this.hidePricing = hidden;
    this.hideNotes = hidden;
  }

  toggleAll() {
    this.setAll(!this.allHidden);
  }
}

export const privacyState = new PrivacyState();

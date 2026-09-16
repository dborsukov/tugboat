/**
 * The frameless window's titlebar is app chrome, not page content: it stays
 * visible above every overlay backdrop (see the layer scale in `app.css`), so
 * interacting with it must not count as interacting "outside" an open dialog,
 * sheet or alert dialog.
 *
 * `onInteractOutside` handler that keeps overlays open while the user works the
 * window controls. Wrapped into every overlay content component, so a component
 * that needs its own logic chains it explicitly:
 *
 * ```svelte
 * onInteractOutside={(e) => {
 *   handleWindowControlsInteractOutside(e);
 *   if (e.defaultPrevented) return;
 *   // ...component specific handling
 * }}
 * ```
 */
export function handleWindowControlsInteractOutside(event: Event) {
  if (isWindowControlsInteraction(event.target)) event.preventDefault();
}

export const WINDOW_CONTROLS_ATTR = "data-window-controls";

/** Whether an outside interaction came from the window controls. */
export function isWindowControlsInteraction(target: EventTarget | null) {
  return (
    target instanceof Element &&
    target.closest(`[${WINDOW_CONTROLS_ATTR}]`) !== null
  );
}

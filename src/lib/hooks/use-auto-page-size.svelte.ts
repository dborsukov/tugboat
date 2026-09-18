import { browser } from "$app/environment";

type UseAutoPageSizeOptions = {
  /** Minimum number of rows to show */
  minPageSize?: number;
};

// Space reserved for a classic (non-overlay) scrollbar. The fitted content is
// kept at least this much shorter than the viewport, so a scrollbar can never
// appear/disappear at an exact-fit boundary its toggling would otherwise
// change the content width, re-wrap row text and flip the page size back and forth.
const SCROLLBAR_RESERVE = 8;

/** The nearest ancestor that actually scrolls, falling back to the document. */
export function findScrollContainer(el: HTMLElement): HTMLElement {
  let node: HTMLElement | null = el.parentElement;
  while (node) {
    const overflowY = getComputedStyle(node).overflowY;
    if (
      overflowY === "auto" ||
      overflowY === "scroll" ||
      overflowY === "overlay"
    ) {
      return node;
    }
    node = node.parentElement;
  }
  return document.documentElement;
}

/**
 * Vertical space that stays below `el` inside `scrollContainer`: the height of
 * its following siblings (e.g. a pagination bar) plus the bottom padding of
 * every ancestor up to and including the scroll container.
 *
 * Rows fitted into the room this measurement leaves free end exactly where the
 * scroll container stops scrolling. Ancestor padding is the part that is easy
 * to forget: the page layout pads its content at the bottom (so a card never
 * touches the window edge), and without it counted here the fitted content
 * would overflow by exactly that padding and grow a scrollbar.
 */
export function measureBelow(
  el: HTMLElement,
  scrollContainer: HTMLElement,
): number {
  let total = 0;
  let node: HTMLElement | null = el;
  while (node) {
    for (
      let sibling = node.nextElementSibling;
      sibling;
      sibling = sibling.nextElementSibling
    ) {
      if (sibling instanceof HTMLElement) total += sibling.offsetHeight;
    }
    // Annotated on purpose: inferring this would be circular through the
    // `node = ancestor` assignment below.
    const ancestor: HTMLElement | null = node.parentElement;
    if (!ancestor) break;
    // getComputedStyle resolves percentage padding to a used pixel value,
    // which Element.style.paddingBottom does not.
    total += Number.parseFloat(getComputedStyle(ancestor).paddingBottom) || 0;
    if (ancestor === scrollContainer) break;
    node = ancestor;
  }
  return total;
}

/**
 * Computes how many fixed-height rows fit into the visible space, so a list
 * fills the page with as many items as can fit.
 *
 * The bound element is the (content-sized) rows container. Its own height
 * tracks its content, so the available space is measured from the nearest
 * scroll container instead, which represents the actual viewport area.
 *
 * ## Usage
 * ```svelte
 * <script lang="ts">
 *   const autoPageSize = new UseAutoPageSize({ minPageSize: 5 });
 *
 *   let rows = $derived(...);
 *   let currentPage = $state(1);
 *
 *   let visible = $derived(
 *     rows.slice((currentPage - 1) * autoPageSize.pageSize, currentPage * autoPageSize.pageSize),
 *   );
 *
 *   // Re-measure whenever the underlying rows change (data load, search, ...).
 *   // Key off the FULL count, not the paginated slice, so a change from, say,
 *   // 137 to 40 rows (both above pageSize) still triggers a re-measure.
 *   $effect(() => {
 *     rows.length;
 *     if (rows.length > 0) autoPageSize.scheduleMeasure();
 *   });
 *
 *   // Clamp the page after the fitted size changes.
 *   $effect(() => {
 *     const pageCount = Math.max(1, Math.ceil(rows.length / autoPageSize.pageSize));
 *     if (currentPage > pageCount) currentPage = pageCount;
 *   });
 * </script>
 *
 * <div bind:this={autoPageSize.container} class="border border-border rounded">
 *   {#each visible as row (row.id)}<Row {row} />{/each}
 * </div>
 * <Pagination.Root count={rows.length} perPage={autoPageSize.pageSize} bind:page={currentPage} />
 * ```
 */
export class UseAutoPageSize {
  /** Bind the rows container to this with `bind:this`. */
  container = $state<HTMLElement | null>(null);
  #pageSize = $state(10);
  #minPageSize: number;
  #measure: (() => void) | undefined = undefined;
  #rafId: number | undefined = undefined;
  #lastCandidate: number | undefined = undefined;

  constructor({ minPageSize = 5 }: Partial<UseAutoPageSizeOptions> = {}) {
    this.#minPageSize = minPageSize;
    // Start at a sane default (at least the requested minimum) so the first
    // paint doesn't briefly render fewer rows than the caller allows.
    this.#pageSize = Math.max(10, this.#minPageSize);

    // The container's own height tracks its content, so we observe the scroll
    // container (its height is the real viewport space) plus the window and
    // re-measure when the rendered rows change via scheduleMeasure().
    $effect(() => {
      const container = this.container;
      if (!container || !browser) return;

      const measure = () => {
        if (this.#rafId !== undefined) cancelAnimationFrame(this.#rafId);
        this.#rafId = requestAnimationFrame(() => {
          this.#rafId = undefined;
          this.#compute(container);
        });
      };
      this.#measure = measure;

      const resizeObserver = new ResizeObserver(measure);
      resizeObserver.observe(findScrollContainer(container));
      // Also observe the column that holds the container, so a sibling below
      // it (e.g. the pagination bar wrapping taller) re-triggers a measure.
      if (container.parentElement instanceof HTMLElement) {
        resizeObserver.observe(container.parentElement);
      }
      window.addEventListener("resize", measure);

      return () => {
        resizeObserver.disconnect();
        window.removeEventListener("resize", measure);
        if (this.#rafId !== undefined) cancelAnimationFrame(this.#rafId);
        this.#measure = undefined;
      };
    });
  }

  // Fit as many rows as the viewport space below the container allows. The
  // estimate must be independent of how many rows are currently rendered:
  // rows must be uniform (no first:/last: padding variants), and the height is
  // sampled from ONE row via getBoundingClientRect().height exact
  // (fractional), unlike offsetHeight which rounds each row to an integer and
  // accumulates up to ~0.5px × N of error, which at an exact-fit boundary
  // makes the fitted content actually overflow (the last item flickering in
  // and out). Recomputations are requestAnimationFrame-coalesced and only
  // applied when the computed size actually differs, so there is no feedback loop.
  #compute(container: HTMLElement) {
    const rows = [...container.children].filter(
      (child): child is HTMLElement => child instanceof HTMLElement,
    );
    if (rows.length === 0) return;
    // Sample a non-first row (index 1 when available): it carries the same
    // divide-y border as the rest of the middle/last rows, so its height is
    // the true per-row height of the list.
    const sampleRow = rows[Math.min(1, rows.length - 1)];
    const rowHeight = sampleRow.getBoundingClientRect().height;
    // Guard against a zero/near-zero height (mount animation, font-load flash,
    // display:contents, an image collapsing the row, ...) which would divide
    // to Infinity.
    if (!Number.isFinite(rowHeight) || rowHeight <= 0) return;

    const scrollContainer = findScrollContainer(container);
    const containerRect = container.getBoundingClientRect();
    const scrollRect = scrollContainer.getBoundingClientRect();
    // Adding scrollTop keeps the offset correct regardless of scroll position.
    const offsetFromContainerTop =
      containerRect.top - scrollRect.top + scrollContainer.scrollTop;
    // Reserve the space taken by everything below the container inside the
    // scroll container (the pagination bar that follows the list in the same
    // column, page padding) plus a scrollbar reserve so the fitted content
    // never overflows by a hair.
    const availableHeight =
      scrollContainer.clientHeight -
      offsetFromContainerTop -
      measureBelow(container, scrollContainer) -
      SCROLLBAR_RESERVE;
    if (availableHeight <= 0) return;
    // A filler element (e.g. an empty state) is not a row, never measure against it.
    if (rowHeight >= availableHeight) return;
    const candidate = Math.max(
      this.#minPageSize,
      Math.floor(availableHeight / rowHeight),
    );
    // Stability filter: apply the first measurement right away, but after
    // that only re-apply a size that was measured twice in a row. At an
    // exact-fit boundary the measurement can flip between two adjacent sizes.
    // Without this, the rendered rows would toggle back and forth every frame.
    if (
      this.#lastCandidate !== undefined &&
      candidate !== this.#lastCandidate
    ) {
      this.#lastCandidate = candidate;
      return;
    }
    this.#lastCandidate = candidate;
    if (candidate !== this.#pageSize) this.#pageSize = candidate;
  }

  /** The number of rows that fit into the available space. */
  get pageSize() {
    return this.#pageSize;
  }

  /** Re-measure after the rendered rows change (data load, search, page turn). */
  scheduleMeasure() {
    if (!browser || !this.#measure) return;
    this.#measure();
  }
}

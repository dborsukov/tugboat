<script lang="ts" generics="TData, TValue">
  import type { Column } from "@tanstack/table-core";
  import { SvelteSet } from "svelte/reactivity";

  import FacetedFilter from "$lib/components/faceted-filter/faceted-filter.svelte";

  import type { FacetedFilterOptions } from "./faceted-filter-options";

  let {
    column,
    title,
    options,
    disabled = false,
    filterColumn = undefined,
  }: {
    /** Column the filter is drawn next to (and the default home of its value). */
    column: Column<TData, TValue>;
    title: string;
    options: FacetedFilterOptions[];
    disabled?: boolean;
    /**
     * Column the value is actually stored on and counted from, when that is not
     * the one being drawn. A table that sorts by a joined name but has to filter
     * by the underlying id keeps both columns and points this at the id.
     */
    filterColumn?: Column<TData, unknown>;
  } = $props();

  const target = $derived(filterColumn ?? column);

  /*
   * Counts keyed the way the options are. TanStack always accumulates an
   * accessor's value as a set, so an accessor that returns `["1"]` (as
   * `arrIncludesSome` requires) is counted under that array rather than under
   * `"1"` the same string the option carries. Re-keying by the value's own
   * string form is what lets the counts line up for both accessor shapes.
   */
  const facets = $derived.by(() => {
    const counts = new Map<string, number>();
    for (const [value, count] of target?.getFacetedUniqueValues() ?? []) {
      const key = String(value);
      counts.set(key, (counts.get(key) ?? 0) + count);
    }
    return counts;
  });

  const selected = $derived(
    new SvelteSet(target?.getFilterValue() as string[]),
  );
</script>

<FacetedFilter
  {title}
  {options}
  {selected}
  {facets}
  {disabled}
  onValueChange={(values) =>
    target?.setFilterValue(values.length ? values : undefined)}
/>

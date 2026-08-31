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
  }: {
    column: Column<TData, TValue>;
    title: string;
    options: FacetedFilterOptions[];
    disabled?: boolean;
  } = $props();

  const facets = $derived(column?.getFacetedUniqueValues());
  const selected = $derived(
    new SvelteSet(column?.getFilterValue() as string[]),
  );
</script>

<FacetedFilter
  {title}
  {options}
  {selected}
  {facets}
  {disabled}
  onValueChange={(values) =>
    column?.setFilterValue(values.length ? values : undefined)}
/>

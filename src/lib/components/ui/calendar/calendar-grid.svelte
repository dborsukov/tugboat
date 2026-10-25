<script lang="ts">
  import { Calendar as CalendarPrimitive } from "bits-ui";

  import { cn } from "$lib/utils.js";

  let {
    ref = $bindable(null),
    class: className,
    ...restProps
  }: CalendarPrimitive.GridProps = $props();
</script>

<CalendarPrimitive.Grid
  bind:ref
  data-slot="calendar-grid"
  class={cn("flex w-full border-collapse flex-col", className)}
  {...restProps}
/>

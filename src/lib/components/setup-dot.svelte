<script lang="ts">
  import { cn } from "$lib/utils";

  let { class: className = "" }: { class?: string } = $props();
</script>

<span
  aria-hidden="true"
  data-setup-dot
  class={cn(
    "inline-block size-2 shrink-0 rounded-full bg-amber-500",
    className,
  )}
></span>

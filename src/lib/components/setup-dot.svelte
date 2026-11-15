<script lang="ts">
  import { cn } from "$lib/utils";

  let {
    class: className = "",
    label = undefined,
  }: { class?: string; label?: string } = $props();
</script>

<span
  role={label ? "img" : undefined}
  aria-label={label}
  aria-hidden={label ? undefined : "true"}
  data-setup-dot
  class={cn(
    "inline-block size-2 shrink-0 rounded-full bg-amber-500",
    className,
  )}
></span>

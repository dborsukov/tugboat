<script lang="ts">
  import { cn } from "$lib/utils";

  type MeterSegment = {
    share: number;
    class?: string;
    label?: string;
  };

  let {
    segments,
    class: className,
  }: { segments: MeterSegment[]; class?: string } = $props();

  function clamp(share: number) {
    return Math.min(100, Math.max(0, share));
  }
</script>

<div
  class={cn("flex h-3 w-full overflow-hidden rounded-full bg-muted", className)}
  role="presentation"
>
  {#each segments as segment, index (index)}
    <div
      class={cn("h-full", segment.class)}
      style="width: {clamp(segment.share)}%"
      title={segment.label}
    ></div>
  {/each}
</div>

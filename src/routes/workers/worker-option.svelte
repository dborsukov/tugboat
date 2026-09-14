<script lang="ts">
  import type { Worker } from "$lib/bindings";
  import { cn } from "$lib/utils";

  import { workerFullName, workerShortName } from "./labels";

  let {
    worker,
    short = false,
    class: className,
  }: {
    worker: Worker;
    short?: boolean;
    class?: string;
  } = $props();
</script>

<div class={cn("flex min-w-0 items-baseline gap-2", className)}>
  <span class={short ? "shrink-0 whitespace-nowrap" : "min-w-0 truncate"}>
    {short ? workerShortName(worker) : workerFullName(worker)}
  </span>
  {#if worker.position}
    <span class="min-w-0 truncate text-sm text-muted-foreground">
      {worker.position}
    </span>
  {/if}
  {#if !worker.isActive}
    <span class="shrink-0 text-xs whitespace-nowrap text-muted-foreground">
      (неактивний)
    </span>
  {/if}
</div>

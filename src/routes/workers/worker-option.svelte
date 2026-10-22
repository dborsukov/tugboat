<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";

  import type { Worker } from "$lib/bindings";
  import { cn } from "$lib/utils";

  import ClientAvatar from "../clients/client-avatar.svelte";
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

<div class={cn("flex min-w-0 items-center gap-2", className)}>
  <ClientAvatar name={workerFullName(worker)} class="size-6 shrink-0" />
  <div class="flex min-w-0 items-baseline gap-2">
    <span class={short ? "shrink-0 whitespace-nowrap" : "min-w-0 truncate"}>
      {short ? workerShortName(worker) : workerFullName(worker)}
    </span>
    {#if worker.position}
      <span class="min-w-0 truncate text-sm text-muted-foreground">
        {worker.position}
      </span>
    {/if}
    {#if !worker.isActive}
      <Badge variant="destructive" class="shrink-0 self-center"
        >неактивний</Badge
      >
    {/if}
  </div>
</div>

<script lang="ts">
  import type { Worker } from "$lib/bindings";

  import ClientAvatar from "../clients/client-avatar.svelte";
  import { workerFullName } from "./labels";

  let { worker }: { worker: Worker } = $props();

  const name = $derived(workerFullName(worker));
</script>

<div class="flex min-w-0 items-center gap-2">
  <ClientAvatar {name} class="size-6 shrink-0" />
  <span class="truncate">{name}</span>
</div>

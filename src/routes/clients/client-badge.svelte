<script lang="ts">
  import type { Client } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import { cn } from "$lib/utils";

  import ClientAvatar from "./client-avatar.svelte";

  interface Props {
    client?: Client;
    clientId?: number;
    iconSize?: string;
    reverse?: boolean;
    class?: string;
  }
  let {
    client,
    clientId,
    iconSize = "size-7",
    reverse = false,
    class: className,
    ...restProps
  }: Props = $props();

  let localClient = $derived(
    client ?? appStore.clients.find((c) => c.id === clientId),
  );
</script>

{#if localClient}
  <div class={cn("flex gap-2 items-center", className)} {...restProps}>
    <ClientAvatar name={localClient.name} class={cn(iconSize)} />
    {localClient.name}
  </div>
{/if}

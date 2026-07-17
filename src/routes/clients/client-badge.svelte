<script lang="ts">
  import type { Client } from "$lib/bindings";

  import { cn } from "$lib/utils";

  import ClientAvatar from "./client-avatar.svelte";

  interface Props {
    client: Client;
    iconSize?: string;
    reverse?: boolean;
    class?: string;
  }
  let {
    client,
    iconSize = "size-7",
    reverse = false,
    class: className,
    ...restProps
  }: Props = $props();
</script>

<div class={cn("flex gap-2 items-center", className)} {...restProps}>
  {#if reverse}
    {client.name}
    <ClientAvatar name={client.name} class={cn(iconSize)} />
  {:else}
    <ClientAvatar name={client.name} class={cn(iconSize)} />
    {client.name}
  {/if}
</div>

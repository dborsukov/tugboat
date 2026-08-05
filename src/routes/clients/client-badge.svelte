<script lang="ts">
  import { cn } from "$lib/utils";

  import ClientAvatar from "./client-avatar.svelte";

  let {
    name,
    iconSize = "size-7",
    reverse = false,
    class: className,
    ...restProps
  }: {
    name: string;
    iconSize?: string;
    reverse?: boolean;
    class?: string;
  } = $props();
</script>

<div class={cn("flex gap-2 items-center", className)} {...restProps}>
  <ClientAvatar {name} class={cn(iconSize)} />
  {name}
</div>

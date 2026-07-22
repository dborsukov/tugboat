<script lang="ts">
  import type { Snippet } from "svelte";

  import { cn } from "$lib/utils";

  import * as Card from "$lib/components/ui/card/index";

  let {
    children,
    class: className = "",
  }: {
    children: Snippet;
    class?: string;
  } = $props();
</script>

<Card.Root class={cn("flex flex-col text-md", className)}>
  <Card.Content>
    {@render children()}
  </Card.Content>
</Card.Root>

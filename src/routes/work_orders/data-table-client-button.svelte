<script lang="ts">
  import ArrowDownIcon from "@lucide/svelte/icons/chevron-down";
  import ArrowUpIcon from "@lucide/svelte/icons/chevron-up";
  import ArrowUpDownIcon from "@lucide/svelte/icons/chevrons-up-down";

  import { Button } from "$lib/components/ui/button/index.js";

  interface Props {
    sortStatus: boolean | string;
    onclick: ((event: unknown) => void) | undefined;
  }

  let { onclick, sortStatus }: Props = $props();
</script>

<Button variant="ghost" class="pl-0" {onclick}>
  Клієнт
  {#if sortStatus === "asc"}
    <ArrowUpIcon />
  {:else if sortStatus === "desc"}
    <ArrowDownIcon />
  {:else}
    <ArrowUpDownIcon />
  {/if}
</Button>

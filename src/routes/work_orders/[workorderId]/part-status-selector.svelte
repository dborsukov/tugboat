<script lang="ts">
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";

  import { Button } from "$lib/components/ui/button";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import Spinner from "$lib/components/ui/spinner/spinner.svelte";

  import { type PartStatus } from "$lib/bindings";
  import { cn } from "$lib/utils";

  import { partStatusStylesRecord, statuses } from "./part-status";

  let {
    status = $bindable(),
    allowedStatuses = statuses,
  }: {
    status: PartStatus;
    allowedStatuses?: PartStatus[];
  } = $props();

  let loading = $state(false);

  const styles = $derived(partStatusStylesRecord[status]);
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger>
    {#snippet child({ props })}
      <Button
        variant="outline"
        class={cn(
          loading && "opacity-75 pointer-events-none",
          styles.base,
          styles.highlighted,
          styles.border,
        )}
        {...props}
      >
        <styles.icon />
        <p>{styles.text}</p>
        {#if loading}
          <Spinner />
        {:else}
          <ChevronDownIcon />
        {/if}
      </Button>
    {/snippet}
  </DropdownMenu.Trigger>
  <DropdownMenu.Content class="w-max whitespace-nowrap" align="end">
    <DropdownMenu.RadioGroup bind:value={status}>
      {#each allowedStatuses as s}
        {@const localStyles = partStatusStylesRecord[s]}
        <DropdownMenu.RadioItem
          value={s}
          class={cn(
            localStyles.base,
            localStyles.highlighted,
            "rounded-none",
            localStyles.rounding,
          )}
        >
          <localStyles.icon />
          <p>{localStyles.text}</p>
        </DropdownMenu.RadioItem>
      {/each}
    </DropdownMenu.RadioGroup>
  </DropdownMenu.Content>
</DropdownMenu.Root>

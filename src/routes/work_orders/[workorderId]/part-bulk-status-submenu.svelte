<script lang="ts">
  import TagIcon from "@lucide/svelte/icons/tag";

  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

  import type { PartStatus } from "$lib/bindings";
  import { cn } from "$lib/utils";

  import { partStatusStylesRecord, statuses } from "./part-status";

  let {
    onStatusChange,
  }: {
    onStatusChange: (status: PartStatus) => void;
  } = $props();
</script>

<DropdownMenu.Sub>
  <DropdownMenu.SubTrigger>
    <TagIcon />
    Змінити статус
  </DropdownMenu.SubTrigger>
  <DropdownMenu.SubContent class="w-55 whitespace-nowrap">
    {#each statuses as s}
      {@const localStyles = partStatusStylesRecord[s]}
      <DropdownMenu.Item
        class={cn(
          localStyles.base,
          localStyles.highlighted,
          "rounded-none",
          localStyles.rounding,
        )}
        onclick={() => onStatusChange(s)}
      >
        <localStyles.icon />
        <p>{localStyles.text}</p>
      </DropdownMenu.Item>
    {/each}
  </DropdownMenu.SubContent>
</DropdownMenu.Sub>

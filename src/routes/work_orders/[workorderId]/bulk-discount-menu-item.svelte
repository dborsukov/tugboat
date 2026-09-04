<script lang="ts">
  import PercentIcon from "@lucide/svelte/icons/percent";

  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

  let {
    onclick,
  }: {
    onclick: () => void;
  } = $props();
</script>

<DropdownMenu.Item {onclick}>
  <PercentIcon />
  Встановити знижку
</DropdownMenu.Item>

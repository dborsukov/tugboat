<script lang="ts">
  import TruckIcon from "@lucide/svelte/icons/truck";

  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

  let { onclick }: { onclick: () => void } = $props();
</script>

<DropdownMenu.Item {onclick}>
  <TruckIcon />
  Змінити постачальника
</DropdownMenu.Item>

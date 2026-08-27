<script lang="ts">
  import { Checkbox } from "$lib/components/ui/checkbox";

  let {
    checked = false,
    indeterminate = false,
    onCheckedChange,
    title = "",
  }: {
    checked?: boolean;
    indeterminate?: boolean;
    onCheckedChange?: (checked: boolean) => void;
    title?: string;
  } = $props();
</script>

<div class="flex items-center justify-center px-2">
  <Checkbox
    {checked}
    {indeterminate}
    {title}
    aria-label={title}
    {onCheckedChange}
  />
</div>

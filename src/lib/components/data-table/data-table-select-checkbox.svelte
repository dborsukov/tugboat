<script lang="ts">
  import { Checkbox } from "$lib/components/ui/checkbox";

  let {
    checked = false,
    indeterminate = false,
    onCheckedChange,
    title = "",
    checkboxClass: className,
  }: {
    checked?: boolean;
    indeterminate?: boolean;
    onCheckedChange?: (checked: boolean) => void;
    title?: string;
    checkboxClass?: string;
  } = $props();
</script>

<div class="flex items-center justify-center">
  <Checkbox
    {checked}
    {indeterminate}
    {title}
    aria-label={title}
    class={className}
    {onCheckedChange}
  />
</div>

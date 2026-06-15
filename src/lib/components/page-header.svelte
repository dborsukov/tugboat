<script lang="ts">
  let { children } = $props();
</script>

<div
  class="h-16 w-full border-b sticky top-0 bg-background/70 backdrop-blur-md"
>
  {@render children?.()}
</div>

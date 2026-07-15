<script lang="ts">
  let { children } = $props();
</script>

<div
  class="flex h-16 w-full border-b sticky top-0 bg-background/70 backdrop-blur-md"
>
  <div class="w-full max-w-7xl mx-auto">
    {@render children?.()}
  </div>
</div>

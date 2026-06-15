<script lang="ts">
  import { goto } from "$app/navigation";

  import Button from "$lib/components/ui/button/button.svelte";
</script>

<div class="w-full h-full flex flex-col items-center justify-center gap-4">
  <h2 class="text-2xl">Клієнта не знайдено</h2>
  <Button variant="outline" onclick={() => goto("/clients")}
    >Повернутись до списку</Button
  >
</div>

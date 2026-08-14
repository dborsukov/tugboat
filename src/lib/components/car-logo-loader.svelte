<script lang="ts">
  import { cn } from "$lib/utils";

  let { slug, class: className }: { slug: string; class?: string } = $props();

  let imageSrc = $state("");
  let notFound = $state(false);
  let isLoading = $state(true);

  $effect(() => {
    isLoading = true;
    const targetSrc = `/car_brand_logos/${slug}.png`;

    const testImg = new Image();
    testImg.src = targetSrc;

    testImg.onload = () => {
      imageSrc = targetSrc;
      isLoading = false;
    };

    testImg.onerror = () => {
      notFound = true;
      isLoading = false;
    };
  });
</script>

<div class={cn("aspect-square overflow-hidden h-full", className)}>
  {#if !isLoading}
    {#if notFound}
      <div
        class="flex items-center justify-center w-full h-full text-xl font-bold"
      >
        ?
      </div>
    {:else}
      <img
        src={imageSrc}
        alt={slug}
        class="max-w-full max-h-full object-contain! aspect-square"
      />
    {/if}
  {:else}
    Loading...
  {/if}
</div>

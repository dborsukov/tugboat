<script>
  let { slug } = $props();

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

<div class="aspect-square overflow-hidden h-full">
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

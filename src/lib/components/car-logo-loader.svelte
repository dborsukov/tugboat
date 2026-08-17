<script lang="ts">
  import { cn } from "$lib/utils";

  import Spinner from "./ui/spinner/spinner.svelte";

  let { slug, class: className }: { slug: string; class?: string } = $props();

  let imageSrc = $state("");
  let notFound = $state(false);
  let isLoading = $state(true);

  const MAX_RETRIES = 3;
  const RETRY_DELAY_MS = 250;

  $effect(() => {
    let cancelled = false;
    let retryCount = 0;
    let retryTimeout: ReturnType<typeof setTimeout>;

    // Reset state for the new slug otherwise a previous failure/image
    // can stick around if this instance is reused with a new slug.
    isLoading = true;
    notFound = false;
    imageSrc = "";

    const targetSrc = `/car_brand_logos/${slug}.png`;

    function attemptLoad() {
      const testImg = new Image();

      // Attach handlers BEFORE setting src. If the image is already
      // cached, some webviews can fire 'load' almost immediately
      // attaching after 'src' risks missing that first event.
      testImg.onload = () => {
        if (cancelled) return;
        imageSrc = targetSrc;
        isLoading = false;
      };

      testImg.onerror = () => {
        if (cancelled) return;

        if (retryCount < MAX_RETRIES) {
          retryCount++;
          // Cache-bust on retry in case the failed request got cached
          // (relevant right after a hard refresh while the asset
          // protocol/dev server is still spinning up).
          retryTimeout = setTimeout(() => {
            if (!cancelled) attemptLoad();
          }, RETRY_DELAY_MS * retryCount);
        } else {
          notFound = true;
          isLoading = false;
        }
      };

      testImg.src =
        retryCount === 0 ? targetSrc : `${targetSrc}?retry=${retryCount}`;
    }

    attemptLoad();

    // Cleanup: ignore any in-flight load/error/retry from this run once
    // the effect re-runs (slug changed) or the component unmounts.
    return () => {
      cancelled = true;
      clearTimeout(retryTimeout);
    };
  });
</script>

<div class={cn("aspect-square overflow-hidden h-full", className)}>
  {#if notFound}
    <div
      class="flex items-center justify-center w-full h-full text-xl font-bold"
    >
      ?
    </div>
  {:else if isLoading}
    <div class="flex items-center justify-center w-full h-full">
      <Spinner />
    </div>
  {:else}
    <img
      src={imageSrc}
      alt={slug}
      class="logo-img max-w-full max-h-full object-contain! aspect-square"
    />
  {/if}
</div>

<style>
  /* Apply white rim glow to logos when using dark theme */
  :global(.dark) .logo-img {
    filter: drop-shadow(0 0 1px rgba(255, 255, 255, 0.5))
      drop-shadow(0 0 2px rgba(255, 255, 255, 0.3));
  }
</style>

<script lang="ts">
  import { getAvatarColor, getInitials } from "$lib/avatar";
  import { cn } from "$lib/utils";

  import Spinner from "./ui/spinner/spinner.svelte";

  let {
    slug,
    name = "",
    class: className,
  }: { slug: string; name?: string; class?: string } = $props();

  const hasIcon = $derived(slug.trim().length > 0);
  const initials = $derived(getInitials(name));
  const colors = $derived(getAvatarColor(name));

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

    if (!hasIcon) {
      isLoading = false;
      return;
    }

    const targetSrc = `/car_brand_logos/${slug}.png`;

    function attemptLoad() {
      const testImg = new Image();

      // Attach handlers BEFORE setting src. If the image is already
      // cached, some webviews can fire 'load' almost immediately
      // attaching after 'src' risks missing that first event.
      testImg.onload = () => {
        if (cancelled) return;

        const show = () => {
          if (cancelled) return;
          imageSrc = targetSrc;
          isLoading = false;
        };

        /*
         * Decode before mounting. WebKit can paint the newly inserted <img>
         * before its bitmap is ready, and the drop-shadow stays clipped to
         * the image box until the later decode repaints it (b/236758).
         * Waiting here makes the first paint already have pixels to shadow.
         */
        testImg.decode().then(show, show);
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

<div class={cn("aspect-square h-full @container", className)}>
  {#if !hasIcon}
    <div
      class={cn(
        // Circle, matching the avatar primitive the client badges use.
        "flex items-center justify-center w-full h-full rounded-full font-semibold select-none",
        colors.bg,
        colors.text,
      )}
      title={name}
    >
      <span class="text-[45cqw] leading-none">{initials}</span>
    </div>
  {:else if notFound}
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
    <div class="logo-glow flex items-center justify-center w-full h-full">
      <img
        src={imageSrc}
        alt={name || slug}
        class="logo-img max-w-full max-h-full object-contain!"
      />
    </div>
  {/if}
</div>

<style>
  /*
   * Apply the white rim glow to the image's wrapper, not to the <img> itself.
   * WebKit 16 clips a drop-shadow painted by an image that loads after the
   * element is in the tree, and only a later repaint fixes it
   * (webkit.org/b/236758). Filtering the parent avoids that image-specific
   * repaint path; translateZ(0) is the usual Safari workaround for
   * drop-shadows that are clipped on their first paint.
   */
  :global(.dark) .logo-glow {
    transform: translateZ(0);
    overflow: visible;
    filter: drop-shadow(0 0 1px rgba(255, 255, 255, 0.5))
      drop-shadow(0 0 2px rgba(255, 255, 255, 0.3));
  }
</style>

<script lang="ts">
  import "flag-icons/css/flag-icons.min.css";

  import { plateTypeMeta, type PlateType } from "$lib/plate";
  import { cn } from "$lib/utils";

  let {
    value,
    plateType = "ua",
    country = null,
    size = "default",
    plain = false,
    class: className,
    title,
  }: {
    value: string;
    plateType?: PlateType | null;
    /** ISO country code for foreign plates. */
    country?: string | null;
    /** `sm` matches the 20px year badge, the default is 24px. */
    size?: "default" | "sm";
    /** Flag + text without the filled plate badge */
    plain?: boolean;
    class?: string;
    title?: string;
  } = $props();

  const compact = $derived(size === "sm");
  const flagSize = $derived(compact ? "h-3 w-4" : "h-3.5 w-4.5");
  const meta = $derived(plateTypeMeta(plateType));
  const hasText = $derived((value ?? "").trim() !== "");
  const flagCountry = $derived(
    meta.chipKind === "country_flag"
      ? (country?.toLowerCase() ?? null)
      : meta.chipKind === "ua_flag"
        ? "ua"
        : null,
  );
  const chipTag = $derived(meta.chipKind === "tag" ? meta.chipTag : null);
</script>

{#if hasText}
  {#if plain}
    <span
      class={cn(
        "inline-flex w-fit items-center gap-1.5 whitespace-nowrap",
        className,
      )}
      title={title ?? value}
    >
      {#if flagCountry}
        <span
          class={cn(
            "fi shrink-0 overflow-hidden rounded-[3px]",
            `fi-${flagCountry}`,
            flagSize,
          )}
        ></span>
      {/if}
      {#if chipTag}
        <span
          class="text-[10.5px] font-bold tracking-[0.06em] text-muted-foreground uppercase"
        >
          {chipTag}
        </span>
      {/if}
      <span>{value}</span>
    </span>
  {:else}
    <span
      class={cn(
        "inline-flex w-fit shrink-0 items-center gap-1.5 whitespace-nowrap rounded-2xl bg-muted px-2 text-xs font-medium text-foreground",
        compact ? "h-5" : "h-6",
        className,
      )}
      title={title ?? value}
    >
      {#if flagCountry}
        <span
          class={cn(
            "fi shrink-0 overflow-hidden rounded-[3px]",
            `fi-${flagCountry}`,
            flagSize,
          )}
        ></span>
      {/if}
      {#if chipTag}
        <span
          class="text-[10.5px] font-bold tracking-[0.06em] text-muted-foreground uppercase"
        >
          {chipTag}
        </span>
      {/if}
      <span>{value}</span>
    </span>
  {/if}
{/if}

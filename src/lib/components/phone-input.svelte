<script lang="ts">
  import { countries, TelInput } from "svelte-tel-input";
  import type {
    CountryCode,
    TelInputOptions,
    Props as TelInputProps,
  } from "svelte-tel-input/types";

  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";

  import * as Command from "$lib/components/ui/command/index.js";
  import * as Popover from "$lib/components/ui/popover/index.js";

  import CountryFlag from "$lib/components/country-flag.svelte";

  import { getCountryNameUk } from "$lib/tools";
  import { cn } from "$lib/utils";

  let {
    value = $bindable(""),
    country = $bindable<CountryCode | null>("UA"),
    options,
    class: className,
    ...props
  }: TelInputProps = $props();

  const countryList = $derived(
    [...countries]
      .map((c) => ({ ...c, ukName: getCountryNameUk(c.iso2, c.name) }))
      .sort((a, b) => {
        if (a.iso2 === "UA") return -1;
        if (b.iso2 === "UA") return 1;
        return a.ukName.localeCompare(b.ukName, "uk");
      }),
  );

  const currentCountry = $derived(
    countries.find((c) => c.iso2 === country) ?? null,
  );

  const mergedOptions = $derived<TelInputOptions>({
    spaces: true,
    autoPlaceholder: true,
    ...options,
  });

  let popoverOpen = $state(false);
</script>

<div class="flex items-center gap-1.5">
  <Popover.Popover bind:open={popoverOpen}>
    <Popover.PopoverTrigger
      class="flex h-8 items-center gap-1 rounded-2xl bg-input/50 px-2.5 text-foreground outline-none select-none hover:bg-muted/70 focus-visible:bg-muted/70 focus-visible:ring-3 focus-visible:ring-ring/30 aria-expanded:bg-muted/70"
      aria-label="Оберіть країну"
    >
      <CountryFlag country={currentCountry?.iso2 ?? null} />
      <span class="text-base font-normal tabular-nums md:text-sm">
        +{currentCountry?.dialCode ?? "380"}
      </span>
      <ChevronDownIcon class="size-3.5 opacity-60" />
    </Popover.PopoverTrigger>

    <Popover.PopoverContent align="start" sideOffset={2} class="w-80 p-0">
      <Command.Command>
        <Command.CommandInput placeholder="Пошук країни..." />
        <Command.CommandList>
          <Command.CommandEmpty>Країну не знайдено</Command.CommandEmpty>
          <Command.CommandGroup>
            {#each countryList as c (c.id)}
              <Command.CommandItem
                value={`${c.name} ${c.ukName} +${c.dialCode}`}
                onSelect={() => {
                  country = c.iso2;
                  popoverOpen = false;
                }}
              >
                <CountryFlag country={c.iso2} />
                <span class="flex-1 truncate">{c.ukName}</span>
                <span class="text-muted-foreground tabular-nums">
                  +{c.dialCode}
                </span>
              </Command.CommandItem>
            {/each}
          </Command.CommandGroup>
        </Command.CommandList>
      </Command.Command>
    </Popover.PopoverContent>
  </Popover.Popover>

  <div class="min-w-0 flex-1">
    <TelInput
      {...props}
      bind:value
      bind:country
      options={mergedOptions}
      defaultCountry="UA"
      placeholderFormat="national"
      class={cn(
        // default shadcn Input classes
        "bg-input/50 focus-visible:border-ring focus-visible:ring-ring/30 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive dark:aria-invalid:border-destructive/50 h-8 rounded-2xl border border-transparent px-2.5 py-1 text-base transition-[color,box-shadow] duration-200 file:h-6 file:text-sm file:font-medium focus-visible:ring-3 aria-invalid:ring-3 md:text-sm file:text-foreground placeholder:text-muted-foreground w-full min-w-0 outline-none file:inline-flex file:border-0 file:bg-transparent disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50",
        className,
      )}
    />
  </div>
</div>

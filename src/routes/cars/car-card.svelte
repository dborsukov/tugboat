<script lang="ts">
  import type { ComponentProps } from "svelte";

  import FingerprintIcon from "@lucide/svelte/icons/fingerprint-pattern";
  import HashIcon from "@lucide/svelte/icons/hash";
  import UserIcon from "@lucide/svelte/icons/user";

  import { Badge } from "$lib/components/ui/badge";
  import * as Card from "$lib/components/ui/card";

  import LogoLoader from "$lib/components/car-logo-loader.svelte";

  import type { Car, CarBrand, Client } from "$lib/bindings";
  import { cn } from "$lib/utils";

  import ClientBadge from "../clients/client-badge.svelte";

  interface Props extends ComponentProps<typeof Card.Root> {
    car: Car;
    client: Client;
    brand: CarBrand;
    short?: boolean;
  }

  let {
    car,
    client,
    brand,
    short = false,
    class: className,
    ...restProps
  }: Props = $props();
</script>

<Card.Root
  class={cn(
    "overflow-hidden transition-all duration-200 rounded hover:shadow-lg hover:border-muted-foreground/30",
    className,
  )}
  {...restProps}
>
  <Card.Header
    class="flex flex-row h-15 justify-center items-center gap-4 space-y-0"
  >
    <LogoLoader slug={brand.iconSlug} />
    <div class="flex-1 min-w-0">
      <Card.Title class="text-xl font-semibold tracking-tight truncate">
        {#if brand != undefined}
          {brand.name}
        {:else}
          ?
        {/if}
      </Card.Title>
      <Card.Description class="text-lg tracking-tight mt-0.5">
        {car.model}
      </Card.Description>
    </div>
    <Badge
      variant="outline"
      class="text-sm font-semibold px-2.5 py-3 bg-background shadow-sm"
    >
      {car.year}
    </Badge>
  </Card.Header>

  {#if !short}
    <Card.Content class="grid gap-2 text-sm">
      <div class="flex items-center justify-between border-b border-muted pb-2">
        <span class="text-muted-foreground flex items-center gap-2">
          <UserIcon class="size-4 text-muted-foreground/70" /> Власник
        </span>
        {#if client != undefined}
          <ClientBadge name={client.name} iconSize="size-6" />
        {:else}
          <span class="text-muted-foreground/50 italic text">Не вказано</span>
        {/if}
      </div>

      <div class="flex items-center justify-between border-b border-muted pb-2">
        <span class="text-muted-foreground flex items-center gap-2">
          <HashIcon class="size-4 text-muted-foreground/70" /> Номерний знак
        </span>
        {#if car.licensePlate}
          <span
            class="font-mono font-semibold uppercase tracking-wider bg-muted px-2 py-0.5 rounded text-xs border"
          >
            {car.licensePlate}
          </span>
        {:else}
          <span class="text-muted-foreground/50 italic text">Не вказано</span>
        {/if}
      </div>

      <div class="flex items-center justify-between">
        <span class="text-muted-foreground flex items-center gap-2">
          <FingerprintIcon class="size-4 text-muted-foreground/70" /> VIN-код
        </span>
        {#if car.vin}
          <span
            class="font-mono font-semibold uppercase tracking-wider bg-muted px-2 py-0.5 rounded text-xs border"
          >
            {car.vin}
          </span>
        {:else}
          <span class="text-muted-foreground/50 italic text-xs">Не вказано</span
          >
        {/if}
      </div>
    </Card.Content>
  {/if}
</Card.Root>

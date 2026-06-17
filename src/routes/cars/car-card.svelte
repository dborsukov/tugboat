<script lang="ts">
  import FingerprintIcon from "@lucide/svelte/icons/fingerprint-pattern";
  import HashIcon from "@lucide/svelte/icons/hash";
  import UserIcon from "@lucide/svelte/icons/user";

  import type { Car } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import { Badge } from "$lib/components/ui/badge";
  import * as Card from "$lib/components/ui/card";

  import LogoLoader from "./logo-loader.svelte";

  let { car, short = false }: { car: Car; short?: boolean } = $props();
  let owner = $derived(appStore.clients.find((c) => c.id === car.client_id));
  let brand = $derived(appStore.car_brands.find((b) => b.id === car.brand_id));
  let slug = $derived(brand != undefined ? brand.icon_slug : "");
</script>

<Card.Root
  class="w-full max-w-full overflow-hidden transition-all duration-200 rounded hover:shadow-lg hover:border-muted-foreground/30"
>
  <Card.Header
    class="flex flex-row h-15 justify-center items-center gap-4 space-y-0"
  >
    <LogoLoader {slug} />
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
        {#if owner != undefined}
          <span class="max-w-45 truncate" title={owner.name}>
            {owner.name}
          </span>
        {:else}
          <span class="text-muted-foreground/50 italic text">Не вказано</span>
        {/if}
      </div>

      <div class="flex items-center justify-between border-b border-muted pb-2">
        <span class="text-muted-foreground flex items-center gap-2">
          <HashIcon class="size-4 text-muted-foreground/70" /> Реєстрація
        </span>
        {#if car.registration}
          <span
            class="font-mono font-bold uppercase tracking-wider bg-muted px-2 py-0.5 rounded text-xs border"
          >
            {car.registration}
          </span>
        {:else}
          <span class="text-muted-foreground/50 italic text">Не вказано</span>
        {/if}
      </div>

      <div class="flex items-center justify-between">
        <span class="text-muted-foreground flex items-center gap-2">
          <FingerprintIcon class="size-4 text-muted-foreground/70" /> VIN
        </span>
        {#if car.vin}
          <span
            class="font-mono font-bold uppercase tracking-wider bg-muted px-2 py-0.5 rounded text-xs border"
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

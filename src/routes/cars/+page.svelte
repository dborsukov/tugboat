<script lang="ts">
  import { compareItems, rankItem } from "@tanstack/match-sorter-utils";

  import PlusIcon from "@lucide/svelte/icons/plus";

  import { appStore } from "$lib/stores/app.svelte";

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as Pagination from "$lib/components/ui/pagination/index.js";

  import PageLayout from "$lib/components/page-layout.svelte";

  import CarCard from "./car-card.svelte";
  import CarMutationSheet from "./car-mutation-sheet.svelte";

  let currentPage = $state(1);
  let perPage = $state(15);
  let searchQuery = $state("");
  let isSheetOpen = $state(false);

  let filteredCars = $derived.by(() => {
    const query = searchQuery.trim();
    if (!query) return appStore.cars;

    return appStore.cars
      .map((car) => {
        const brand = appStore.car_brands.find((b) => b.id === car.brand_id);
        const modelName = `${brand?.name ?? ""} ${car.model ?? ""}`.trim();
        const owner = appStore.clients.find((c) => c.id === car.client_id);

        const modelRank = rankItem(modelName, query);
        const plateRank = rankItem(car.registration ?? "", query);
        const vinRank = rankItem(car.vin ?? "", query);
        const ownerRank = rankItem(owner?.name ?? "", query);

        // keep whichever field matched best for this car
        const bestRank = [modelRank, plateRank, vinRank, ownerRank].reduce(
          (best, rank) => (rank.rank > best.rank ? rank : best),
        );

        return {
          car,
          bestRank,
          passed:
            modelRank.passed ||
            plateRank.passed ||
            vinRank.passed ||
            ownerRank.passed,
        };
      })
      .filter(({ passed }) => passed)
      .sort((a, b) => compareItems(a.bestRank, b.bestRank))
      .map(({ car }) => car);
  });

  let paginatedCars = $derived(
    filteredCars.slice((currentPage - 1) * perPage, currentPage * perPage),
  );

  function handleSearchInput() {
    currentPage = 1;
  }
</script>

<PageLayout>
  {#snippet header()}
    <div class="h-full px-4 flex items-center gap-2">
      <Input
        placeholder="Пошук за моделю, номером, VIN-кодом або власником..."
        class="w-full"
        bind:value={searchQuery}
        oninput={handleSearchInput}
      />
      <Button variant="outline" onclick={() => (isSheetOpen = !isSheetOpen)}>
        Додати <PlusIcon />
      </Button>
      <CarMutationSheet bind:open={isSheetOpen} />
    </div>
  {/snippet}
  {#if paginatedCars.length === 0}
    <div class="flex h-40 items-center justify-center text-muted-foreground">
      Автомобілей не знайдено
    </div>
  {:else}
    <div
      class="grid gap-4 p-4 grid-cols-[repeat(auto-fit,minmax(200px,405px))] justify-center"
    >
      {#each paginatedCars as car (car.id)}
        <a href="/cars/{car.id}" class="w-full">
          <CarCard {car} />
        </a>
      {/each}
    </div>
  {/if}
  {#snippet footer()}
    <div class="p-4 flex items-center justify-center">
      <Pagination.Root
        count={filteredCars.length}
        {perPage}
        bind:page={currentPage}
      >
        {#snippet children({ pages, currentPage })}
          <Pagination.Content>
            <Pagination.Item>
              <Pagination.PrevButton />
            </Pagination.Item>

            {#each pages as page (page.key)}
              {#if page.type === "ellipsis"}
                <Pagination.Item>
                  <Pagination.Ellipsis />
                </Pagination.Item>
              {:else}
                <Pagination.Item>
                  <Pagination.Link {page} isActive={currentPage === page.value}>
                    {page.value}
                  </Pagination.Link>
                </Pagination.Item>
              {/if}
            {/each}

            <Pagination.Item>
              <Pagination.NextButton />
            </Pagination.Item>
          </Pagination.Content>
        {/snippet}
      </Pagination.Root>
    </div>
  {/snippet}
</PageLayout>

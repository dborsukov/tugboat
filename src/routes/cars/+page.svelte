<script lang="ts">
  import { compareItems, rankItem } from "@tanstack/match-sorter-utils";
  import { createQueries } from "@tanstack/svelte-query";

  import SortIcon from "@lucide/svelte/icons/arrow-up-down";
  import PlusIcon from "@lucide/svelte/icons/plus";

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as Pagination from "$lib/components/ui/pagination/index.js";
  import * as Select from "$lib/components/ui/select/index.js";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import { clientsQueryOptions } from "../clients/queries";
  import CarCard from "./car-card.svelte";
  import CarMutationSheet from "./car-mutation-sheet.svelte";
  import { carBrandsQueryOptions, carsQueryOptions } from "./queries";
  import { buildCarListItems } from "./types";

  type SortOption = "date-desc" | "date-asc";

  const sortLabels: Record<SortOption, string> = {
    "date-desc": "Спочатку нові",
    "date-asc": "Спочатку старі",
  };

  let currentPage = $state(1);
  let perPage = $state(15);
  let searchQuery = $state("");
  let isSheetOpen = $state(false);
  let sortBy = $state<SortOption>("date-desc");
  let searchInputEl = $state<HTMLInputElement | null>(null);

  const combinedQueries = createQueries(() => ({
    queries: [
      carsQueryOptions(),
      clientsQueryOptions(),
      carBrandsQueryOptions(),
    ],
    combine: (results) => {
      return {
        cars: results[0],
        clients: results[1],
        carBrands: results[2],

        isReady: results.every((r) => r.isSuccess),
        isLoading: results.some((r) => r.isPending),
        isError: results.some((r) => r.isError),
        firstError: results.find((r) => r.isError)?.error,

        refetchAll: () => {
          results.forEach((r) => r.refetch());
        },
      };
    },
  }));

  let carListItems = $derived(
    combinedQueries.isReady
      ? buildCarListItems(
          combinedQueries.cars.data!,
          combinedQueries.clients.data!,
          combinedQueries.carBrands.data!,
        )
      : [],
  );

  let filteredCarListItems = $derived.by(() => {
    const query = searchQuery.trim();
    if (!query) return carListItems;

    return carListItems
      .map((cli) => {
        const modelRank = rankItem(`${cli.brand.name} ${cli.car.model}`, query);
        const plateRank = rankItem(cli.car.licensePlate ?? "", query);
        const vinRank = rankItem(cli.car.vin ?? "", query);
        const clientRank = rankItem(cli.client.name, query);

        const bestRank = [modelRank, plateRank, vinRank, clientRank].reduce(
          (best, rank) => (rank.rank > best.rank ? rank : best),
        );

        return {
          car: cli,
          bestRank,
          passed:
            modelRank.passed ||
            plateRank.passed ||
            vinRank.passed ||
            clientRank.passed,
        };
      })
      .filter(({ passed }) => passed)
      .sort((a, b) => compareItems(a.bestRank, b.bestRank))
      .map(({ car }) => car);
  });

  let sortedCarListItems = $derived.by(() => {
    const sorted = [...filteredCarListItems];

    switch (sortBy) {
      case "date-asc":
        sorted.sort(
          (a, b) =>
            new Date(a.car.createdAt).getTime() -
            new Date(b.car.createdAt).getTime(),
        );
        break;
      case "date-desc":
      default:
        sorted.sort(
          (a, b) =>
            new Date(b.car.createdAt).getTime() -
            new Date(a.car.createdAt).getTime(),
        );
        break;
    }

    return sorted;
  });

  let paginatedCarListItems = $derived(
    sortedCarListItems.slice(
      (currentPage - 1) * perPage,
      currentPage * perPage,
    ),
  );

  function handleSearchInput() {
    currentPage = 1;
  }

  function handleSortChange() {
    currentPage = 1;
  }

  let controlsDisabled = $derived(
    !combinedQueries.isReady ||
      (paginatedCarListItems.length === 0 && !searchQuery),
  );

  $effect(() => searchInputEl?.focus());
</script>

<PageLayout>
  {#snippet header()}
    <div class="h-full flex items-center gap-2">
      <Input
        bind:ref={searchInputEl}
        bind:value={searchQuery}
        placeholder="Пошук за моделлю, номером, VIN-кодом або власником..."
        oninput={handleSearchInput}
        disabled={controlsDisabled}
        class="w-full"
      />
      <Select.Root
        bind:value={sortBy}
        type="single"
        onValueChange={handleSortChange}
        disabled={controlsDisabled}
      >
        <Select.Trigger class="w-45 shrink-0">
          <SortIcon />
          {sortLabels[sortBy]}
        </Select.Trigger>
        <Select.Content>
          <Select.Item value="date-desc">{sortLabels["date-desc"]}</Select.Item>
          <Select.Item value="date-asc">{sortLabels["date-asc"]}</Select.Item>
        </Select.Content>
      </Select.Root>
      <Button variant="outline" onclick={() => (isSheetOpen = !isSheetOpen)}>
        Додати <PlusIcon />
      </Button>
      <CarMutationSheet bind:open={isSheetOpen} />
    </div>
  {/snippet}
  {#if combinedQueries.isError}
    <FetchErrorDisplay
      message={combinedQueries.firstError?.message}
      onRetry={combinedQueries.refetchAll}
      class="mt-20"
    />
  {:else if combinedQueries.isLoading}
    <div
      class="grid gap-4 p-4 grid-cols-[repeat(auto-fit,minmax(200px,405px))] justify-center"
    >
      <Skeleton class="w-full h-55" />
      <Skeleton class="w-full h-55" />
      <Skeleton class="w-full h-55" />
      <Skeleton class="w-full h-55" />
      <Skeleton class="w-full h-55" />
      <Skeleton class="w-full h-55" />
    </div>
  {:else if paginatedCarListItems.length === 0}
    <p class="text-center text-muted-foreground mt-20">
      Автомобілей не знайдено
    </p>
  {:else}
    <div
      class="grid gap-4 p-4 grid-cols-[repeat(auto-fit,minmax(200px,405px))] justify-center"
    >
      {#each paginatedCarListItems as cli (cli.car.id)}
        <a href="/cars/{cli.car.id}" class="w-full">
          <CarCard car={cli.car} client={cli.client} brand={cli.brand} />
        </a>
      {/each}
    </div>
  {/if}
  {#snippet footer()}
    <div class="p-4 flex items-center justify-center">
      <Pagination.Root
        count={filteredCarListItems.length}
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

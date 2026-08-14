<script lang="ts">
  import { compareItems, rankItem } from "@tanstack/match-sorter-utils";

  import { goto } from "$app/navigation";

  import SortIcon from "@lucide/svelte/icons/arrow-up-down";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import PlusIcon from "@lucide/svelte/icons/plus";

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as Pagination from "$lib/components/ui/pagination/index.js";
  import * as Select from "$lib/components/ui/select/index.js";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import CarLogoLoader from "$lib/components/car-logo-loader.svelte";
  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import CarMutationSheet from "./car-mutation-sheet.svelte";
  import { createCarDetailsListQuery } from "./queries";

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

  const carDetailsQuery = createCarDetailsListQuery();
  const carDetailsList = $derived(carDetailsQuery.data ?? []);

  let filteredCarListItems = $derived.by(() => {
    if (!carDetailsQuery.isSuccess) return [];
    const query = searchQuery.trim();
    if (!query) return carDetailsList;

    return carDetailsList
      .map((cli) => {
        const modelRank = rankItem(
          `${cli.carDto?.brand?.name} ${cli.carDto?.car?.model}`,
          query,
        );
        const plateRank = rankItem(cli.carDto?.car?.licensePlate ?? "", query);
        const vinRank = rankItem(cli.carDto?.car?.vin ?? "", query);
        const clientRank = rankItem(cli.client?.name, query);

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
            new Date(a.carDto.car.createdAt).getTime() -
            new Date(b.carDto.car.createdAt).getTime(),
        );
        break;
      case "date-desc":
      default:
        sorted.sort(
          (a, b) =>
            new Date(b.carDto.car.createdAt).getTime() -
            new Date(a.carDto.car.createdAt).getTime(),
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

  function carSecondaryLine(cli: (typeof carDetailsList)[number]) {
    return [
      cli.carDto?.car?.year,
      cli.carDto?.car?.vin,
      cli.carDto?.car?.licensePlate,
    ]
      .filter(Boolean)
      .join(" · ");
  }

  let controlsDisabled = $derived(
    !carDetailsQuery.isSuccess ||
      (paginatedCarListItems.length === 0 && !searchQuery),
  );

  $effect(() => searchInputEl?.focus());
</script>

<PageLayout contentWidthClass="max-w-5xl" headerWidthClass="max-w-5xl">
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
      <Button
        variant="outline"
        onclick={() => (isSheetOpen = !isSheetOpen)}
        disabled={!carDetailsQuery.isSuccess}
      >
        Додати <PlusIcon />
      </Button>
      <CarMutationSheet bind:open={isSheetOpen} />
    </div>
  {/snippet}

  {#if carDetailsQuery.isError}
    <FetchErrorDisplay
      message={carDetailsQuery.error?.message}
      onRetry={carDetailsQuery.refetch}
      class="mt-10"
    />
  {:else if carDetailsQuery.isLoading}
    <div
      class="mt-4 flex flex-col border border-border rounded divide-y divide-border"
    >
      <Skeleton class="w-full h-16" />
      <Skeleton class="w-full h-16" />
      <Skeleton class="w-full h-16" />
      <Skeleton class="w-full h-16" />
      <Skeleton class="w-full h-16" />
    </div>
  {:else if paginatedCarListItems.length === 0}
    {#if searchQuery}
      <div
        class="mt-4 p-4 flex border border-border rounded bg-card items-center justify-center"
      >
        <p>Пошук не повернув результатів</p>
      </div>
    {:else}
      <div
        class="mt-4 p-4 flex flex-col border border-border rounded bg-card items-center justify-center"
      >
        <p>Автомобілей не знайдено</p>
        <p class="text-muted-foreground">
          Натисніть "Додати +" щоб додати новий автомобіль
        </p>
      </div>
    {/if}
  {:else}
    <div
      class="mt-4 flex flex-col border border-border rounded divide-y divide-border bg-card"
    >
      {#each paginatedCarListItems as cli (cli.carDto?.car?.id)}
        <Button
          variant="ghost"
          class="w-full h-auto justify-start rounded-none px-5 py-3 first:pt-4 last:pb-4"
          onclick={async () => await goto(`/cars/${cli.carDto?.car?.id}`)}
        >
          <div class="flex w-full items-center gap-3.5">
            <CarLogoLoader slug={cli.carDto?.brand?.iconSlug} class="h-12" />
            <div class="flex-1 min-w-0 text-left flex flex-col gap-0.5">
              <p class="truncate text-lg font-medium">
                {cli.carDto?.brand?.name}
                {cli.carDto?.car?.model}
              </p>
              <p class="truncate text-muted-foreground font-mono">
                {carSecondaryLine(cli)}
              </p>
            </div>
            <span class="shrink-0 text-muted-foreground">
              {cli.client?.name}
            </span>
            <ChevronRightIcon class="size-4 shrink-0 text-muted-foreground" />
          </div>
        </Button>
      {/each}
    </div>

    <div class="p-4 flex items-center justify-center">
      <Pagination.Root
        count={filteredCarListItems.length}
        {perPage}
        bind:page={currentPage}
      >
        {#snippet children({ pages, currentPage })}
          <Pagination.Content class="gap-5">
            <Pagination.Item>
              <Pagination.PrevButton
                class="border-none shadow-none text-muted-foreground hover:text-foreground"
              />
            </Pagination.Item>

            {#each pages as page (page.key)}
              {#if page.type === "ellipsis"}
                <Pagination.Item>
                  <Pagination.Ellipsis />
                </Pagination.Item>
              {:else}
                <Pagination.Item>
                  <Pagination.Link
                    {page}
                    isActive={currentPage === page.value}
                    class="border-none shadow-none {currentPage === page.value
                      ? 'font-semibold text-foreground'
                      : 'text-muted-foreground'}"
                  >
                    {page.value}
                  </Pagination.Link>
                </Pagination.Item>
              {/if}
            {/each}

            <Pagination.Item>
              <Pagination.NextButton
                class="border-none shadow-none text-muted-foreground hover:text-foreground"
              />
            </Pagination.Item>
          </Pagination.Content>
        {/snippet}
      </Pagination.Root>
    </div>
  {/if}
</PageLayout>

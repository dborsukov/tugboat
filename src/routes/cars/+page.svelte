<script lang="ts">
  import { compareItems, rankItem } from "@tanstack/match-sorter-utils";

  import { goto } from "$app/navigation";

  import ArchiveIcon from "@lucide/svelte/icons/archive";
  import SortIcon from "@lucide/svelte/icons/arrow-up-down";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import PlusIcon from "@lucide/svelte/icons/plus";

  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as Pagination from "$lib/components/ui/pagination/index.js";
  import * as Select from "$lib/components/ui/select/index.js";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import ArchivedBadge from "$lib/components/archived-badge.svelte";
  import CarLogoLoader from "$lib/components/car-logo-loader.svelte";
  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import LicensePlate from "$lib/components/license-plate.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import { UseAutoPageSize } from "$lib/hooks/use-auto-page-size.svelte";

  import ClientAvatar from "../clients/client-avatar.svelte";
  import CarMutationSheet from "./car-mutation-sheet.svelte";
  import { createCarDetailsListQuery } from "./queries";

  type SortOption = "date-desc" | "date-asc";

  const sortLabels: Record<SortOption, string> = {
    "date-desc": "Спочатку нові",
    "date-asc": "Спочатку старі",
  };

  let currentPage = $state(1);
  let searchQuery = $state("");
  let isSheetOpen = $state(false);
  let sortBy = $state<SortOption>("date-desc");
  let showArchived = $state(false);
  let searchInputEl = $state<HTMLInputElement | null>(null);

  const carDetailsQuery = createCarDetailsListQuery();
  const autoPageSize = new UseAutoPageSize({ minPageSize: 5 });

  let visibleCarDetailsList = $derived.by(() => {
    const cars = carDetailsQuery.data ?? [];
    return showArchived
      ? cars.filter((car) => car.carDto?.car?.archivedAt != null)
      : cars.filter((car) => car.carDto?.car?.archivedAt == null);
  });

  const archivedCarsCount = $derived(
    (carDetailsQuery.data ?? []).filter(
      (car) => car.carDto?.car?.archivedAt != null,
    ).length,
  );

  let filteredCarListItems = $derived.by(() => {
    if (!carDetailsQuery.isSuccess) return [];
    const query = searchQuery.trim();
    if (!query) return visibleCarDetailsList;

    return visibleCarDetailsList
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
      (currentPage - 1) * autoPageSize.pageSize,
      currentPage * autoPageSize.pageSize,
    ),
  );

  function handleSearchInput() {
    currentPage = 1;
  }

  function handleSortChange() {
    currentPage = 1;
  }

  let controlsDisabled = $derived(
    !carDetailsQuery.isSuccess || (carDetailsQuery.data?.length ?? 0) === 0,
  );

  $effect(() => searchInputEl?.focus());

  // Re-measure the fitted page size whenever the underlying rows change (data
  // load, search). Key off the FULL filtered count, not the paginated slice,
  // so a change from e.g. 137 to 40 rows (both above pageSize) re-measures.
  $effect(() => {
    filteredCarListItems.length;
    if (carDetailsQuery.isSuccess && paginatedCarListItems.length > 0) {
      autoPageSize.scheduleMeasure();
    }
  });

  // Keep the current page valid after the fitted size changes (window resize).
  $effect(() => {
    const pageCount = Math.max(
      1,
      Math.ceil(filteredCarListItems.length / autoPageSize.pageSize),
    );
    if (currentPage > pageCount) currentPage = pageCount;
  });
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
        variant={showArchived ? "default" : "outline"}
        onclick={() => {
          showArchived = !showArchived;
          currentPage = 1;
        }}
        disabled={controlsDisabled}
        class="shrink-0"
      >
        <ArchiveIcon />
        Архів
        {#if archivedCarsCount > 0}
          <span class="text-muted-foreground">{archivedCarsCount}</span>
        {/if}
      </Button>
      <Button
        variant="outline"
        onclick={() => (isSheetOpen = !isSheetOpen)}
        disabled={!carDetailsQuery.isSuccess || showArchived}
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
  {:else}
    <div class="pt-4 flex flex-col">
      <div
        bind:this={autoPageSize.container}
        class="border border-border rounded bg-card flex flex-col @container"
      >
        {#if carDetailsQuery.isLoading}
          <Skeleton class="w-full h-16 shrink-0" />
          <Skeleton class="w-full h-16 shrink-0" />
          <Skeleton class="w-full h-16 shrink-0" />
          <Skeleton class="w-full h-16 shrink-0" />
          <Skeleton class="w-full h-16 shrink-0" />
        {:else if paginatedCarListItems.length === 0}
          {#if searchQuery}
            <div class="p-4 flex items-center justify-center">
              <p>Пошук не повернув результатів</p>
            </div>
          {:else if showArchived}
            <div class="p-4 flex flex-col items-center justify-center">
              <p>Архівних автомобілів не знайдено</p>
            </div>
          {:else}
            <div class="p-4 flex flex-col items-center justify-center">
              <p>Автомобілей не знайдено</p>
              <p class="text-muted-foreground">
                Натисніть "Додати +" щоб додати новий автомобіль
              </p>
            </div>
          {/if}
        {:else}
          {#each paginatedCarListItems as cli (cli.carDto?.car?.id)}
            <div class="border-b border-border last:border-b-0">
              <Button
                variant="ghost"
                class="w-full h-auto justify-start rounded-none px-5 py-3 shrink-0 {cli
                  .carDto?.car?.archivedAt
                  ? 'opacity-60'
                  : ''}"
                onclick={async () => await goto(`/cars/${cli.carDto?.car?.id}`)}
              >
                <div class="flex w-full items-center gap-3.5">
                  <CarLogoLoader
                    slug={cli.carDto?.brand?.iconSlug ?? ""}
                    name={cli.carDto?.brand?.name}
                    class="h-10"
                  />
                  <div class="flex-1 min-w-0 text-left flex flex-col gap-1">
                    <div class="flex items-center gap-2">
                      <p class="truncate text-base font-semibold">
                        {cli.carDto?.brand?.name}
                        {cli.carDto?.car?.model}
                      </p>
                      {#if cli.carDto?.car?.archivedAt}
                        <ArchivedBadge date={cli.carDto?.car?.archivedAt} />
                      {/if}
                    </div>
                    <p
                      class="flex min-w-0 items-center gap-1.5 truncate text-muted-foreground"
                    >
                      {#if cli.carDto?.car?.year}
                        <Badge
                          variant="secondary"
                          class="h-5 rounded-2xl bg-muted px-2 text-xs font-medium text-muted-foreground"
                          >{cli.carDto.car.year}</Badge
                        >
                      {/if}
                      {#if cli.carDto?.car?.licensePlate}
                        <LicensePlate
                          value={cli.carDto.car.licensePlate}
                          size="sm"
                        />
                      {/if}
                    </p>
                  </div>
                  <span
                    class="flex shrink-0 items-center gap-2 text-muted-foreground @max-[500px]:hidden"
                  >
                    <ClientAvatar
                      name={cli.client?.name}
                      class="size-7 shrink-0"
                    />
                    <span>{cli.client?.name}</span>
                  </span>
                  <ChevronRightIcon
                    class="size-4 shrink-0 text-muted-foreground"
                  />
                </div>
              </Button>
            </div>
          {/each}
        {/if}
      </div>

      {#if paginatedCarListItems.length > 0}
        <div class="p-4 flex items-center justify-center sticky bottom-0">
          <Pagination.Root
            count={filteredCarListItems.length}
            perPage={autoPageSize.pageSize}
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
                        class="border-none shadow-none {currentPage ===
                        page.value
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
    </div>
  {/if}
</PageLayout>

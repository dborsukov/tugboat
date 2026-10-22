<script lang="ts">
  import { compareItems, rankItem } from "@tanstack/match-sorter-utils";
  import type { Component } from "svelte";
  import { SvelteSet } from "svelte/reactivity";

  import { goto } from "$app/navigation";

  import ArchiveIcon from "@lucide/svelte/icons/archive";
  import SortIcon from "@lucide/svelte/icons/arrow-up-down";
  import BriefcaseIcon from "@lucide/svelte/icons/briefcase";
  import BuildingIcon from "@lucide/svelte/icons/building-2";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import FunnelXIcon from "@lucide/svelte/icons/funnel-x";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import UserIcon from "@lucide/svelte/icons/user";

  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as Pagination from "$lib/components/ui/pagination/index.js";
  import * as Select from "$lib/components/ui/select/index.js";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import ArchivedBadge from "$lib/components/archived-badge.svelte";
  import type { FacetedFilterOptions } from "$lib/components/data-table-faceted-filter/faceted-filter-options";
  import FacetedFilter from "$lib/components/faceted-filter/faceted-filter.svelte";
  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";
  import PhoneDisplay from "$lib/components/phone-display.svelte";

  import type { ClientType } from "$lib/bindings";
  import { UseAutoPageSize } from "$lib/hooks/use-auto-page-size.svelte";
  import { pluralizeUk } from "$lib/tools";

  import ClientAvatar from "./client-avatar.svelte";
  import ClientMutationSheet from "./client-mutation-sheet.svelte";
  import { createClientsQuery } from "./queries";

  type SortOption = "date-desc" | "date-asc" | "name-asc" | "name-desc";

  const sortLabels: Record<SortOption, string> = {
    "date-desc": "Спочатку нові",
    "date-asc": "Спочатку старі",
    "name-asc": "За ім'ям (А-Я)",
    "name-desc": "За ім'ям (Я-А)",
  };

  const clientTypeLabels: Record<ClientType, string> = {
    individual: "Фізична особа",
    fop: "ФОП",
    company: "Організація",
  };

  const clientTypeIcons: Record<ClientType, Component> = {
    individual: UserIcon,
    fop: BriefcaseIcon,
    company: BuildingIcon,
  };

  const clientTypeFilterOptions: FacetedFilterOptions[] = (
    ["individual", "fop", "company"] as ClientType[]
  ).map((value) => ({
    label: clientTypeLabels[value],
    value,
    icon: clientTypeIcons[value],
  }));

  let currentPage = $state(1);
  let searchQuery = $state("");
  let isSheetOpen = $state(false);
  let sortBy = $state<SortOption>("date-desc");
  let showArchived = $state(false);
  let searchInputEl = $state<HTMLInputElement | null>(null);
  let selectedClientTypes = new SvelteSet<string>();

  const clientsQuery = createClientsQuery();
  const autoPageSize = new UseAutoPageSize({ minPageSize: 5 });

  let visibleClients = $derived.by(() => {
    const clients = clientsQuery.isSuccess ? clientsQuery.data : [];
    return showArchived
      ? clients.filter((client) => client.archivedAt != null)
      : clients.filter((client) => client.archivedAt == null);
  });

  const archivedClientsCount = $derived(
    (clientsQuery.data ?? []).filter((client) => client.archivedAt != null)
      .length,
  );

  const clientTypeFacets = $derived.by(() => {
    const counts = new Map<string, number>();
    for (const client of visibleClients) {
      counts.set(client.clientType, (counts.get(client.clientType) ?? 0) + 1);
    }
    return counts;
  });

  let filteredClients = $derived.by(() => {
    let clients = visibleClients;
    const query = searchQuery.trim();

    if (selectedClientTypes.size > 0) {
      clients = clients.filter((client) =>
        selectedClientTypes.has(client.clientType),
      );
    }
    if (!query) return clients;

    return clients
      .map((client) => {
        const fields = [
          { rank: rankItem(client.name, query), passed: false },
          { rank: rankItem(client.phone ?? "", query), passed: false },
          { rank: rankItem(client.email ?? "", query), passed: false },
          {
            rank: rankItem(client.contactPhone ?? "", query),
            passed: false,
          },
        ];
        const best = fields.reduce((a, b) =>
          a.rank.rank > b.rank.rank ? a : b,
        );
        return {
          client,
          bestRank: best.rank,
          passed: fields.some((f) => f.rank.passed),
        };
      })
      .filter(({ passed }) => passed)
      .sort((a, b) => compareItems(a.bestRank, b.bestRank))
      .map(({ client }) => client);
  });

  let sortedClients = $derived.by(() => {
    const sorted = [...filteredClients];

    switch (sortBy) {
      case "name-asc":
        sorted.sort((a, b) => a.name.localeCompare(b.name, "uk"));
        break;
      case "name-desc":
        sorted.sort((a, b) => b.name.localeCompare(a.name, "uk"));
        break;
      case "date-asc":
        sorted.sort(
          (a, b) =>
            new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime(),
        );
        break;
      case "date-desc":
      default:
        sorted.sort(
          (a, b) =>
            new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime(),
        );
        break;
    }

    return sorted;
  });

  let paginatedClients = $derived(
    sortedClients.slice(
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
    !clientsQuery.isSuccess || (clientsQuery.data?.length ?? 0) === 0,
  );

  function handleClearTypeFilter() {
    selectedClientTypes.clear();
    currentPage = 1;
  }

  $effect(() => searchInputEl?.focus());

  // Re-measure the fitted page size whenever the underlying rows change (data
  // load, search). Key off the FULL filtered count, not the paginated slice,
  // so a change from e.g. 137 to 40 rows (both above pageSize) re-measures.
  $effect(() => {
    filteredClients.length;
    if (clientsQuery.isSuccess && paginatedClients.length > 0) {
      autoPageSize.scheduleMeasure();
    }
  });

  // Keep the current page valid after the fitted size changes (window resize).
  $effect(() => {
    const pageCount = Math.max(
      1,
      Math.ceil(filteredClients.length / autoPageSize.pageSize),
    );
    if (currentPage > pageCount) currentPage = pageCount;
  });
</script>

<PageLayout contentWidthClass="max-w-5xl" headerWidthClass="max-w-5xl">
  {#snippet header()}
    <div class="h-full flex items-center gap-2">
      <Input
        bind:ref={searchInputEl}
        placeholder="Пошук за ім'ям або телефоном..."
        class="w-full"
        bind:value={searchQuery}
        oninput={handleSearchInput}
        disabled={controlsDisabled}
      />
      <Select.Root
        type="single"
        bind:value={sortBy}
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
          <Select.Item value="name-asc">{sortLabels["name-asc"]}</Select.Item>
          <Select.Item value="name-desc">{sortLabels["name-desc"]}</Select.Item>
        </Select.Content>
      </Select.Root>
      <FacetedFilter
        title="Тип клієнта"
        options={clientTypeFilterOptions}
        selected={selectedClientTypes}
        facets={clientTypeFacets}
        disabled={controlsDisabled}
      />
      {#if selectedClientTypes.size > 0}
        <Button
          variant="ghost"
          onclick={handleClearTypeFilter}
          class="px-2 py-4"
          disabled={controlsDisabled}
        >
          <FunnelXIcon />
          Прибрати фільтри
        </Button>
      {/if}
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
        {#if archivedClientsCount > 0}
          <span class="text-muted-foreground">{archivedClientsCount}</span>
        {/if}
      </Button>
      <Button
        variant="outline"
        onclick={() => (isSheetOpen = !isSheetOpen)}
        disabled={!clientsQuery.isSuccess || showArchived}
      >
        Додати <PlusIcon />
      </Button>
      <ClientMutationSheet bind:open={isSheetOpen} />
    </div>
  {/snippet}

  {#if clientsQuery.isError}
    <FetchErrorDisplay
      message={clientsQuery.error?.message}
      onRetry={clientsQuery.refetch}
      class="mt-10"
    />
  {:else}
    <div class="pt-4 flex flex-col">
      <div
        bind:this={autoPageSize.container}
        class="border border-border rounded bg-card flex flex-col"
      >
        {#if clientsQuery.isLoading}
          <Skeleton class="w-full h-14 shrink-0" />
          <Skeleton class="w-full h-14 shrink-0" />
          <Skeleton class="w-full h-14 shrink-0" />
          <Skeleton class="w-full h-14 shrink-0" />
          <Skeleton class="w-full h-14 shrink-0" />
        {:else if paginatedClients.length === 0}
          {#if searchQuery}
            <div class="p-4 flex items-center justify-center">
              <p>Пошук не повернув результатів</p>
            </div>
          {:else if selectedClientTypes.size > 0}
            <div class="p-4 flex items-center justify-center">
              <p>За вибраним типом клієнтів не знайдено</p>
            </div>
          {:else if showArchived}
            <div class="p-4 flex flex-col items-center justify-center">
              <p>Архівних клієнтів не знайдено</p>
            </div>
          {:else}
            <div class="p-4 flex flex-col items-center justify-center">
              <p>Клієнтів не знайдено</p>
              <p class="text-muted-foreground">
                Натисніть "Додати +" щоб додати нового клієнта
              </p>
            </div>
          {/if}
        {:else}
          {#each paginatedClients as client (client.id)}
            <div class="border-b border-border last:border-b-0">
              <Button
                variant="ghost"
                class="w-full h-auto justify-start rounded-none px-5 py-3 shrink-0 {client.archivedAt
                  ? 'opacity-60'
                  : ''}"
                onclick={async () => await goto(`/clients/${client.id}`)}
              >
                <div class="flex w-full items-center gap-3.5">
                  <ClientAvatar name={client?.name} class="size-10 shrink-0" />
                  <div class="flex min-w-0 flex-1 flex-col gap-0.5">
                    <div class="flex min-w-0 items-center gap-2">
                      <p class="truncate text-left text-base font-semibold">
                        {client?.name}
                      </p>
                      {#if client?.clientType !== "individual"}
                        <Badge
                          variant="secondary"
                          class="h-5 shrink-0 rounded-2xl bg-muted px-2 text-xs font-medium text-muted-foreground"
                        >
                          {clientTypeLabels[client.clientType]}
                        </Badge>
                      {/if}
                      {#if client.archivedAt}
                        <ArchivedBadge date={client.archivedAt} />
                      {/if}
                    </div>
                    <p class="truncate text-left text-sm text-muted-foreground">
                      {client.carCount} авто · {client.workOrderCount}
                      {pluralizeUk(client.workOrderCount, [
                        "замовлення",
                        "замовлення",
                        "замовлень",
                      ])}
                    </p>
                  </div>
                  <PhoneDisplay
                    value={client?.clientType !== "individual"
                      ? (client?.contactPhone ?? client?.phone)
                      : client?.phone}
                    class="shrink-0 text-sm text-muted-foreground select-none!"
                  />
                  <ChevronRightIcon
                    class="size-4 shrink-0 text-muted-foreground"
                  />
                </div>
              </Button>
            </div>
          {/each}
        {/if}
      </div>

      {#if paginatedClients.length > 0}
        <div class="p-4 flex items-center justify-center sticky bottom-0">
          <Pagination.Root
            count={filteredClients.length}
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

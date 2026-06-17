<script lang="ts">
  import CirclePlusIcon from "@lucide/svelte/icons/circle-plus";

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

  let filteredCars = $derived(
    appStore.cars.filter((car) => {
      const query = searchQuery.trim().toLowerCase();
      if (!query) return true;

      const brand = appStore.car_brands.find((b) => b.id === car.brand_id);
      const name = brand?.name + car.model;

      const matchesName = name.toLowerCase().includes(query);
      const matchesReg = car.registration?.toLowerCase().includes(query);
      const matchesVin = car.vin?.toLowerCase().includes(query);

      return matchesName || matchesReg || matchesVin;
    }),
  );

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
      <!-- TODO: implement fuzzy search with highlight -->
      <Input
        placeholder="Пошук за моделю або номером"
        class="w-full"
        bind:value={searchQuery}
        oninput={handleSearchInput}
      />
      <Button onclick={() => (isSheetOpen = !isSheetOpen)}>
        <CirclePlusIcon /> Створити
      </Button>
      <CarMutationSheet bind:open={isSheetOpen} />
    </div>
  {/snippet}
  {#if paginatedCars.length === 0}
    <!-- TODO: center this or replace with shadcn's Empty -->
    <div class="flex h-40 items-center justify-center text-muted-foreground">
      Автомобілей не знайдено
    </div>
  {:else}
    <div class="flex flex-wrap gap-4 p-4">
      {#each paginatedCars as car (car.id)}
        <a href="/cars/{car.id}" class="flex-1 min-w-max">
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

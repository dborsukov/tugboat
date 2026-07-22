<script lang="ts">
  import PlusIcon from "@lucide/svelte/icons/plus";

  import { appStore } from "$lib/stores/app.svelte";

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";

  import DataTable from "$lib/components/data-table.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import { columns } from "./columns";
  import ServiceMutationSheet from "./service-mutation-sheet.svelte";

  let inputValue: string = $state("");
  let isSheetOpen: boolean = $state(false);
</script>

<PageLayout>
  {#snippet header()}
    <div class="h-full px-4 flex items-center gap-2">
      <Input
        placeholder="Пошук за кодом, назвою або нормо-годинами..."
        bind:value={inputValue}
        class="w-full"
      />
      <Button
        variant="outline"
        onclick={() => {
          isSheetOpen = !isSheetOpen;
        }}
      >
        Додати <PlusIcon />
      </Button>
      <ServiceMutationSheet bind:open={isSheetOpen} />
    </div>
  {/snippet}
  <div class="p-4">
    <DataTable
      data={appStore.services}
      {columns}
      bind:searchTerm={inputValue}
      emptyMessage="Сервісів не знайдено..."
      headerClasses={{ name: "w-full" }}
    />
  </div>
</PageLayout>

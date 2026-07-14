<script lang="ts">
  import UserPlusIcon from "@lucide/svelte/icons/circle-plus";

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
        placeholder="Пошук за назвою або ціною..."
        bind:value={inputValue}
        class="w-full"
      />
      <Button
        onclick={() => {
          isSheetOpen = !isSheetOpen;
        }}
      >
        <UserPlusIcon /> Додати
      </Button>
      <ServiceMutationSheet bind:open={isSheetOpen} />
    </div>
  {/snippet}
  <DataTable
    data={appStore.services}
    {columns}
    bind:searchTerm={inputValue}
    emptyMessage="Сервісів не знайдено..."
    headerClasses={{ name: "w-full" }}
  />
</PageLayout>

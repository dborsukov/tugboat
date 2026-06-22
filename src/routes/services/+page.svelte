<script lang="ts">
  import type { Table } from "@tanstack/table-core";

  import UserPlusIcon from "@lucide/svelte/icons/circle-plus";

  import type { Service } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";

  import PageLayout from "$lib/components/page-layout.svelte";

  import { columns } from "./columns";
  import DataTable from "./data-table.svelte";

  // let table: Table<Service>;
</script>

<PageLayout>
  {#snippet header()}
    <div class="h-full px-4 flex items-center gap-2">
      <!-- TODO: implement fuzzy search with highlight -->
      <Input
        placeholder="Пошук за назвою..."
        // value={(table.getColumn("name")?.getFilterValue() as string) ?? ""}
        // onchange={(e) => {
        //   table.getColumn("name")?.setFilterValue(e.currentTarget.value);
        // }}
        // oninput={(e) => {
        //   table.getColumn("name")?.setFilterValue(e.currentTarget.value);
        // }}
        class="w-full"
      />
      <Button>
        <UserPlusIcon /> Додати
      </Button>
      <!-- <ClientMutationSheet bind:open={isSheetOpen} /> -->
    </div>
  {/snippet}
  <DataTable data={appStore.services} {columns} />
</PageLayout>

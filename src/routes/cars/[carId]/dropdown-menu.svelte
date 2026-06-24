<script lang="ts">
  import { toast } from "svelte-sonner";

  import EllipsisIcon from "@lucide/svelte/icons/ellipsis";
  import EditIcon from "@lucide/svelte/icons/square-pen";
  import TrashIcon from "@lucide/svelte/icons/trash";

  import { goto } from "$app/navigation";

  import { commands, type Car } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import { buttonVariants } from "$lib/components/ui/button/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

  import DeleteDialog from "$lib/components/delete-dialog.svelte";

  import CarMutationSheet from "../car-mutation-sheet.svelte";

  interface Props {
    car: Car;
  }
  let { car }: Props = $props();

  let isSheetOpen = $state(false);
  let isDeleteDialogOpen = $state(false);

  async function handleDelete() {
    let res = await commands.deleteCar(car.id);
    if (res.status === "ok") {
      appStore.removeCar(car.id);
      await goto("/cars");
      toast.success("Автомобіль видалено");
    } else {
      toast.error("Виникла помилка", {
        description: res.error.message,
        position: "bottom-center",
      });
    }
  }
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger
    class={buttonVariants({ variant: "outline", size: "icon-sm" })}
  >
    <EllipsisIcon />
  </DropdownMenu.Trigger>
  <DropdownMenu.Content class="w-40" align="end">
    <DropdownMenu.Group>
      <DropdownMenu.Item onSelect={() => (isSheetOpen = true)}>
        Редагувати
        <EditIcon class="ml-auto" />
      </DropdownMenu.Item>
      <DropdownMenu.Item
        // FIXME: figure out how to make icon red on focus(hover)
        // class="text-destructive focus:text-destructive"
        onSelect={() => (isDeleteDialogOpen = true)}
      >
        Видалити
        <TrashIcon class="ml-auto" />
      </DropdownMenu.Item>
    </DropdownMenu.Group>
  </DropdownMenu.Content>
</DropdownMenu.Root>

<CarMutationSheet bind:open={isSheetOpen} {car} />
<DeleteDialog
  bind:open={isDeleteDialogOpen}
  onDelete={handleDelete}
  description="Цю дію неможливо відмінити. Усі пов'язані з цим автомобілем дані будуть видалені."
/>

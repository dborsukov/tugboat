<script lang="ts">
  import { toast } from "svelte-sonner";

  import SheetHost from "$lib/components/sheet/sheet-host.svelte";

  import type { CreateOrEditProps } from "$lib/types";

  import WorkOrderSheetContent from "./work-order-sheet-content.svelte";

  type Props = CreateOrEditProps<{ carId: number }, { workOrderId: string }> & {
    open: boolean;
    isLocked?: boolean;
    onCreated?: (id: string) => void;
  };

  let {
    open = $bindable(false),
    isLocked = false,
    onCreated,
    ...rest
  }: Props = $props();

  const isEdit = $derived(rest.mode === "edit");
</script>

<SheetHost bind:open autofocus>
  <WorkOrderSheetContent
    active={open}
    session={open ? 1 : 0}
    mode={rest.mode}
    initialCarId={rest.mode === "create" ? rest.carId : null}
    workOrderId={rest.mode === "edit" ? rest.workOrderId : null}
    {isLocked}
    allowPicker
    onCancel={() => (open = false)}
    onCreated={(id) => {
      open = false;
      toast.success(isEdit ? "Зміни збережено" : "Наряд-замовлення створено", {
        position: "bottom-left",
      });
      onCreated?.(id);
    }}
  />
</SheetHost>

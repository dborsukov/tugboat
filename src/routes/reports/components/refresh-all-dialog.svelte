<script lang="ts">
  import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js";

  import { pluralizeUk } from "$lib/tools";

  let {
    open = $bindable(false),
    count,
    onConfirm,
  }: {
    open: boolean;
    count: number;
    onConfirm: () => void;
  } = $props();

  const reports = $derived(
    `${count} ${pluralizeUk(count, ["звіт", "звіти", "звітів"])}`,
  );
</script>

<AlertDialog.Root bind:open>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>Оновити {reports}?</AlertDialog.Title>
      <AlertDialog.Description>
        Дані змінилися або файли зникли з диска, тож буде записано нові версії;
        попередні залишаться в журналі. Це може зайняти кілька секунд.
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Action
        onclick={() => {
          open = false;
          onConfirm();
        }}>Оновити всі</AlertDialog.Action
      >
      <AlertDialog.Cancel variant="outline">Відмінити</AlertDialog.Cancel>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>

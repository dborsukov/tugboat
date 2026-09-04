<script lang="ts">
  import Big from "big.js";

  import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { Label } from "$lib/components/ui/label";
  import Spinner from "$lib/components/ui/spinner/spinner.svelte";

  import type { DiscountType } from "$lib/bindings";
  import { pluralizeUk } from "$lib/tools";

  import DiscountField from "../discount-field.svelte";

  interface BulkDiscountPreview {
    belowCost: boolean;
    belowCostNow: boolean;
  }

  let {
    title = "Встановити знижку",
    description = null,
    open = $bindable(false),
    rows = [] as { id: number }[],
    preview,
    applyDiscount,
  }: {
    title?: string;
    description?: string | null;
    open: boolean;
    rows: { id: number }[];
    preview: (
      id: number,
      discountType: DiscountType,
      discountValue: string,
    ) => Promise<BulkDiscountPreview | null>;
    applyDiscount: (
      ids: number[],
      discountType: DiscountType,
      discountValue: string,
    ) => Promise<void>;
  } = $props();

  let discountType = $state<DiscountType>("Fixed");
  let discountValue = $state("");
  let error = $state<string | null>(null);
  let applying = $state(false);

  let belowCostConfirmOpen = $state(false);
  let pendingApply = $state<{
    discountType: DiscountType;
    discountValue: string;
  } | null>(null);
  let belowCostCount = $state(0);

  $effect(() => {
    if (open) {
      discountType = "Fixed";
      discountValue = "";
      error = null;
      applying = false;
      belowCostConfirmOpen = false;
      pendingApply = null;
      belowCostCount = 0;
    }
  });

  function validate(): string | null {
    if (discountType === "None") return null;

    const raw = discountValue.trim().replace(",", ".");
    if (raw === "") return "Вкажіть знижку";

    let num: Big;
    try {
      num = new Big(raw);
    } catch {
      return "Вкажіть знижку";
    }

    if (discountType === "Percentage") {
      if (num.lte(0)) return "Вкажіть знижку";
      if (num.gt(100)) return "Знижка у відсотках має бути в межах 100%";
    } else {
      if (num.lte(0)) return "Вкажіть знижку";
    }
    return null;
  }

  function normalizeValue(): string {
    if (discountType === "None") return "0";
    return discountValue.trim().replace(",", ".");
  }

  async function handleApply() {
    const message = validate();
    error = message;
    if (message !== null) return;

    const type = discountType;
    const value = normalizeValue();
    applying = true;
    try {
      const previews = await Promise.all(
        rows.map((row) => preview(row.id, type, value)),
      );
      let belowCostTotal = 0;
      previews.forEach((quote) => {
        if (quote && quote.belowCost && !quote.belowCostNow) {
          belowCostTotal += 1;
        }
      });
      if (belowCostTotal > 0) {
        belowCostCount = belowCostTotal;
        pendingApply = { discountType: type, discountValue: value };
        belowCostConfirmOpen = true;
        return;
      }
      await submitApply(type, value);
    } finally {
      applying = false;
    }
  }

  async function confirmBelowCostApply() {
    if (pendingApply === null) return;
    belowCostConfirmOpen = false;
    const pending = pendingApply;
    pendingApply = null;
    applying = true;
    try {
      await submitApply(pending.discountType, pending.discountValue);
    } finally {
      applying = false;
    }
  }

  function cancelBelowCostApply() {
    belowCostConfirmOpen = false;
    pendingApply = null;
  }

  async function submitApply(type: DiscountType, value: string): Promise<void> {
    await applyDiscount(
      rows.map((row) => row.id),
      type,
      value,
    );
    open = false;
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="sm:max-w-sm gap-4">
    <Dialog.Header>
      <Dialog.Title>{title}</Dialog.Title>
      {#if description}
        <Dialog.Description>{description}</Dialog.Description>
      {/if}
    </Dialog.Header>
    <div class="flex flex-col gap-1.5">
      <Label>Знижка</Label>
      <DiscountField bind:type={discountType} bind:value={discountValue} />
      {#if error}
        <p class="text-sm text-destructive">{error}</p>
      {/if}
    </div>
    <Dialog.Footer>
      <Button
        variant="outline"
        onclick={() => (open = false)}
        disabled={applying}
      >
        Відмінити
      </Button>
      <Button onclick={handleApply} disabled={applying}>
        {#if applying}
          <Spinner class="size-4" />
        {/if}
        Застосувати
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<AlertDialog.Root bind:open={belowCostConfirmOpen}>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>Ціна нижча за собівартість</AlertDialog.Title>
      <AlertDialog.Description>
        {belowCostCount}{" "}
        {pluralizeUk(belowCostCount, [
          "позиція буде продана",
          "позиції будуть продані",
          "позицій буде продано",
        ])}{" "}
        нижче собівартості. Застосувати знижку?
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel onclick={cancelBelowCostApply}>
        Відмінити
      </AlertDialog.Cancel>
      <AlertDialog.Action onclick={confirmBelowCostApply}>
        Застосувати
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>

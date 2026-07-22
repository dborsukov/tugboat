<script lang="ts">
  import { NumberFormatStyle, NumericFormat } from "svelte-number-format";
  import { toast } from "svelte-sonner";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import { goto } from "$app/navigation";

  import { commands, type WorkOrder } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import { sqidsEncode } from "$lib/misc";

  import { Button } from "$lib/components/ui/button";
  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Sheet from "$lib/components/ui/sheet";
  import { Spinner } from "$lib/components/ui/spinner";

  interface Props {
    open: boolean;
    work_order?: WorkOrder | null;
    car_id?: number | null;
  }

  let {
    open = $bindable(false),
    work_order = null,
    car_id = null,
  }: Props = $props();

  const isEdit = $derived(!!work_order);
  const titleText = $derived(
    isEdit ? "Редагувати наряд-замовлення" : "Створити наряд-замовлення",
  );
  const descText = $derived(
    isEdit
      ? "Змініть дані замовлення тут."
      : "Внесіть дані нового замовлення тут.",
  );
  const formID = "work-order-mutation-form";

  export const workOrderSchema = z.object({
    odometer_in: z
      .number({ error: "Вкажіть пробіг" })
      .nonnegative("Неправильний пробіг")
      .default(0),
    rate: z
      .number({ error: "Вкажіть тариф" })
      .nonnegative("Неправильний тариф")
      .default(0),
  });

  const form = superForm(defaults(zod4(workOrderSchema)), {
    SPA: true,
    validators: zod4Client(workOrderSchema),
    validationMethod: "oninput",
    resetForm: false,
    id: formID,
    async onUpdate({ form: nativeForm }) {
      if (!nativeForm.valid) return;

      const rateInCents = Math.round(nativeForm.data.rate * 100);

      if (isEdit) {
        if (work_order === null) {
          return toast.error("Критична помилка", {
            description:
              'Некоректний стан застосунку("work_order" не ініціалізовано.)',
            position: "bottom-left",
          });
        }
        let res = await commands.updateWorkOrder({
          id: work_order.id,
          car_id: work_order.car_id,
          started_at: work_order.started_at,
          completed_at: work_order.completed_at,
          odometer_in: nativeForm.data.odometer_in,
          customer_complaint: work_order.customer_complaint,
          mechanic_notes: work_order.mechanic_notes,
          status: work_order.status,
          rate: rateInCents,
        });
        if (res.status === "ok") {
          appStore.updateWorkOrder(res.data);
        } else {
          return toast.error("Виникла помилка", {
            description: res.error.message,
            position: "bottom-left",
          });
        }
      } else {
        if (car_id === null) {
          return toast.error("Критична помилка", {
            description:
              'Некоректний стан застосунку("car_id" не ініціалізовано.)',
            position: "bottom-left",
          });
        }
        let new_id = `WO-${sqidsEncode([Date.now()])}`;
        let res = await commands.createWorkOrder({
          id: new_id,
          car_id: car_id,
          created_at: new Date().toISOString(),
          odometer_in: nativeForm.data.odometer_in,
          status: "Received",
          customer_complaint: null,
          mechanic_notes: null,
          rate: rateInCents,
        });
        if (res.status === "ok") {
          appStore.createWorkOrder(res.data);
          await goto(`/work_orders/${new_id}`);
        } else {
          return toast.error("Виникла помилка", {
            description: res.error.message,
            position: "bottom-left",
          });
        }
      }
      open = false;
      form.reset();
    },
  });

  const { form: formData, enhance, submitting } = form;

  $effect(() => {
    if (open) {
      if (work_order) {
        $formData.odometer_in = work_order.odometer_in;
        $formData.rate = work_order.rate / 100;
      } else {
        form.reset();
      }
    }
  });

  function handleCancel() {
    form.reset();
    open = false;
  }
</script>

<Sheet.Root bind:open>
  <Sheet.Content
    side="right"
    showCloseButton={false}
    onEscapeKeydown={(e) => e.preventDefault()}
    onInteractOutside={(e) => e.preventDefault()}
  >
    <Sheet.Header>
      <Sheet.Title>{titleText}</Sheet.Title>
      <Sheet.Description>{descText}</Sheet.Description>
    </Sheet.Header>

    <form
      use:enhance
      id={formID}
      class="flex flex-col p-6 gap-4 overflow-y-auto h-full"
    >
      <Form.Field {form} name="odometer_in">
        <Form.Control>
          {#snippet children({ props })}
            <Label>Одометр(пробіг)</Label>
            <div class="flex gap-2 items-baseline">
              <Input
                type="number"
                min="0"
                {...props}
                bind:value={$formData.odometer_in}
                class="flex-1"
              />
              <p>км.</p>
            </div>
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>
      <Form.Field {form} name="rate">
        <Form.Control>
          {#snippet children({ props })}
            <Label>Тариф</Label>
            <div class="flex gap-2 items-baseline" {...props}>
              <NumericFormat
                bind:value={$formData.rate}
                onkeydown={(e: KeyboardEvent) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    const input = e.currentTarget as HTMLInputElement;
                    input.blur();
                    input.closest("form")?.requestSubmit();
                  }
                }}
                locale="ua-UK"
                options={{
                  formatStyle: NumberFormatStyle.Decimal,
                  currency: "UAH",
                  precision: 2,
                }}
                class="flex-1 bg-input/50 focus-visible:border-ring focus-visible:ring-ring/30 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive dark:aria-invalid:border-destructive/50 h-8 rounded-2xl border border-transparent px-2.5 py-1 text-base transition-[color,box-shadow] duration-200 file:h-6 file:text-sm file:font-medium focus-visible:ring-3 aria-invalid:ring-3 md:text-sm file:text-foreground placeholder:text-muted-foreground w-full min-w-0 outline-none file:inline-flex file:border-0 file:bg-transparent disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50"
              />
              <p>грн./год.</p>
            </div>
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>
    </form>
    <Sheet.Footer>
      <Button type="submit" disabled={$submitting} form={formID}>
        {#if $submitting}<Spinner />{/if}
        {isEdit ? "Зберегти зміни" : "Створити"}
      </Button>
      <Button variant="outline" disabled={$submitting} onclick={handleCancel}>
        Відмінити та закрити
      </Button>
    </Sheet.Footer>
  </Sheet.Content>
</Sheet.Root>

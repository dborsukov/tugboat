<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";
  import { toast } from "svelte-sonner";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import { goto } from "$app/navigation";

  import { Button } from "$lib/components/ui/button";
  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";

  import CurrencyInput from "$lib/components/currency-input.svelte";
  import MutationSheetShell from "$lib/components/mutation-sheet-shell.svelte";

  import { commands, type CreateWorkOrderPayload } from "$lib/bindings";
  import { sqidsEncode, unwrap } from "$lib/tools";

  import {
    createUpdateWorkOrderMutation,
    createWorkOrdersQuery,
    invalidateWorkOrderLists,
  } from "./queries";

  let {
    woId = null,
    carId = null,
    open = $bindable(false),
  }: {
    woId?: string | null;
    carId?: number | null;
    open: boolean;
  } = $props();

  const queryClient = useQueryClient();

  let workOrdersQuery = createWorkOrdersQuery();
  let workOrdersList = $derived(workOrdersQuery.data ?? []);
  let workOrder = $derived(workOrdersList.find((wo) => wo.id === woId) ?? null);

  const isEdit = $derived(!!woId);
  const formID = "work-order-mutation-form";

  export const workOrderSchema = z
    .object({
      odometerIn: z
        .number({ error: "Вкажіть пробіг" })
        .nonnegative("Неправильний пробіг")
        .default(0),
      rate: z
        .number({ error: "Вкажіть тариф" })
        .nonnegative("Неправильний тариф")
        .default(0),
      discountType: z.enum(["Fixed", "Percentage"]).default("Fixed"),
      discount: z.number({ error: "Вкажіть знижку" }).default(0),
    })
    .superRefine((data, ctx) => {
      if (data.discountType === "Fixed") {
        if (data.discount < 0) {
          ctx.addIssue({
            code: "custom",
            path: ["discount"],
            message: "Знижка не може бути від'ємною",
          });
        }
      } else {
        if (data.discount < 0) {
          ctx.addIssue({
            code: "custom",
            path: ["discount"],
            message: "Відсоток не може бути від'ємним",
          });
        }
        if (data.discount > 100) {
          ctx.addIssue({
            code: "custom",
            path: ["discount"],
            message: "Відсоток не може перевищувати 100",
          });
        }
      }
    });

  const createWorkOrderMutation = createMutation(() => ({
    mutationFn: (payload: CreateWorkOrderPayload) =>
      unwrap(commands.createWorkOrder(payload)),
    onSuccess: () => {
      invalidateWorkOrderLists(queryClient);
    },
    onError: (error) => {
      toast.error("Виникла помилка", {
        description: error.message,
        position: "bottom-left",
      });
    },
  }));

  const updateWorkOrderMutation = createUpdateWorkOrderMutation(queryClient);

  const form = superForm(defaults(zod4(workOrderSchema)), {
    SPA: true,
    validators: zod4Client(workOrderSchema),
    validationMethod: "oninput",
    resetForm: false,
    id: formID,
    async onUpdate({ form: nativeForm }) {
      if (!nativeForm.valid) return;

      let new_id = `WO-${sqidsEncode(Date.now())}`;

      const rateInCents = Math.round(nativeForm.data.rate * 100);
      const discountCents =
        nativeForm.data.discountType === "Fixed"
          ? Math.round(nativeForm.data.discount * 100)
          : 0;
      const discountPercent =
        nativeForm.data.discountType === "Percentage"
          ? nativeForm.data.discount
          : 0;

      if (!isEdit && !carId) {
        toast.error("Виникла помилка", {
          description: "Prop 'carId' missing",
          position: "bottom-left",
        });
      }

      const result =
        isEdit && woId
          ? await updateWorkOrderMutation
              .mutateAsync({
                id: workOrder!.id,
                carId: workOrder!.carId,
                odometerIn: nativeForm.data.odometerIn,
                customerComplaint: workOrder!.customerComplaint,
                mechanicNotes: workOrder!.mechanicNotes,
                status: workOrder!.status,
                rateCents: rateInCents,
                discountType: nativeForm.data.discountType,
                discountCents,
                discountPercent,
                startedAt: workOrder!.startedAt,
                completedAt: workOrder!.completedAt,
                paidAt: workOrder!.paidAt,
                paymentMethod: workOrder!.paymentMethod,
              })
              .catch(() => null)
          : await createWorkOrderMutation
              .mutateAsync({
                id: new_id,
                carId: carId!,
                odometerIn: nativeForm.data.odometerIn,
                status: "Received",
                customerComplaint: null,
                mechanicNotes: null,
                rateCents: rateInCents,
                discountType: nativeForm.data.discountType,
                discountCents,
                discountPercent,
              })
              .catch(() => null);

      if (!result) return;
      if (!isEdit) await goto(`/work_orders/${new_id}`);
      open = false;
      form.reset();
    },
  });

  const { form: formData, enhance, submitting } = form;

  $effect(() => {
    if (open) {
      if (workOrder) {
        $formData.odometerIn = workOrder.odometerIn;
        $formData.rate = workOrder.rateCents / 100;
        $formData.discountType = workOrder.discountType;
        $formData.discount =
          workOrder.discountType === "Fixed"
            ? workOrder.discountCents / 100
            : workOrder.discountPercent;
      } else {
        form.reset();
      }
    }
  });
</script>

<MutationSheetShell
  bind:open
  {formID}
  title={isEdit ? "Редагувати наряд-замовлення" : "Створити наряд-замовлення"}
  description={isEdit
    ? "Змініть дані наряду-замовлення"
    : "Внесіть дані нового наряду-замовлення"}
  submitLabel={isEdit ? "Зберегти зміни" : "Створити"}
  submitting={$submitting}
  enhanceAction={enhance}
  onCancel={() => {
    form.reset();
    open = false;
  }}
>
  <Form.Field {form} name="odometerIn">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Одометр(пробіг)</Label>
        <div class="flex gap-2 items-baseline">
          <Input
            type="number"
            min="0"
            {...props}
            bind:value={$formData.odometerIn}
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
          <CurrencyInput
            bind:value={$formData.rate}
            onkeydown={(e: KeyboardEvent) => {
              if (e.key === "Enter") {
                e.preventDefault();
                const input = e.currentTarget as HTMLInputElement;
                input.blur();
                input.closest("form")?.requestSubmit();
              }
            }}
          />
          <p>грн./год.</p>
        </div>
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="discount">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Знижка</Label>
        <div class="flex flex-col gap-2">
          <div class="flex gap-2">
            <Button
              type="button"
              variant={$formData.discountType === "Fixed"
                ? "default"
                : "outline"}
              class="flex-1"
              onclick={() => ($formData.discountType = "Fixed")}
            >
              Фіксована
            </Button>
            <Button
              type="button"
              variant={$formData.discountType === "Percentage"
                ? "default"
                : "outline"}
              class="flex-1"
              onclick={() => ($formData.discountType = "Percentage")}
            >
              Відсоток
            </Button>
          </div>
          <div class="flex gap-2 items-baseline">
            {#if $formData.discountType === "Fixed"}
              <Input
                type="number"
                min="0"
                step="0.01"
                {...props}
                bind:value={$formData.discount}
                class="flex-1"
              />
              <p>грн.</p>
            {:else}
              <Input
                type="number"
                min="0"
                max="100"
                step="0.01"
                {...props}
                bind:value={$formData.discount}
                class="flex-1"
              />
              <p>%</p>
            {/if}
          </div>
        </div>
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>
</MutationSheetShell>

<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";
  import { toast } from "svelte-sonner";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import {
    commands,
    type CreateWorkOrderPartItemPayload,
    type PartStatus,
    type UpdateWorkOrderPartItemPayload,
  } from "$lib/bindings";

  import { unwrap } from "$lib/misc";

  import { Button } from "$lib/components/ui/button";
  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";

  import CurrencyInput from "$lib/components/currency-input.svelte";
  import MutationSheetShell from "$lib/components/mutation-sheet-shell.svelte";

  import {
    createWorkOrderPartItemsQuery,
    invalidateWorkOrderLists,
  } from "../queries";
  import PartStatusSelector from "./part-status-selector.svelte";

  let {
    id = null,
    woId = null,
    open = $bindable(false),
  }: {
    id?: number | null;
    woId?: string | null;
    open: boolean;
  } = $props();

  const queryClient = useQueryClient();

  let partItemsQuery = createWorkOrderPartItemsQuery();
  let partItems = $derived(partItemsQuery.data ?? []);
  let partItem = $derived(partItems.find((c) => c.id === id) ?? null);

  const isEdit = $derived(!!partItem);
  const formID = $derived(
    isEdit
      ? `work-order-part-item-mutation-form-${id}`
      : "work-order-part-item-mutation-form-create",
  );

  const partItemSchema = z.object({
    status: z.string().default("Quoted"),
    code: z.string().nullable(),
    name: z.string().min(1, { error: "Вкажіть назву" }),
    quantity: z
      .number({ error: "Вкажіть кількість" })
      .nonnegative("Неправильна кількість")
      .default(1),
    unitType: z.string().min(1, { error: "Вкажіть одиницю вимірювання" }),
    cost: z
      .number({ error: "Вкажіть ціну" })
      .nonnegative("Неправильна ціна")
      .default(0),
    markup: z
      .number({ error: "Вкажіть націнку у відсотках" })
      .min(0, "Неправильний відсоток")
      .max(100, "Неправильний відсоток")
      .default(0),
  });

  const createWorkOrderPartItemMutation = createMutation(() => ({
    mutationFn: async (payload: CreateWorkOrderPartItemPayload) =>
      unwrap(commands.createWorkOrderPartItem(payload)),
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

  const updateWorkOrderPartItemMutation = createMutation(() => ({
    mutationFn: async (variables: {
      id: number;
      payload: UpdateWorkOrderPartItemPayload;
    }) =>
      unwrap(commands.updateWorkOrderPartItem(variables.id, variables.payload)),
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

  const form = superForm(defaults(zod4(partItemSchema)), {
    SPA: true,
    validators: zod4Client(partItemSchema),
    validationMethod: "oninput",
    resetForm: false,
    // svelte-ignore state_referenced_locally
    id: formID,
    async onUpdate({ form: nativeForm }) {
      if (!nativeForm.valid) return;

      const payload = {
        status: nativeForm.data.status as PartStatus,
        code: nativeForm.data.code,
        name: nativeForm.data.name,
        quantity: nativeForm.data.quantity,
        unitType: nativeForm.data.unitType,
        costCents: nativeForm.data.cost * 100,
        markup: nativeForm.data.markup,
      };

      if (isEdit && id) {
        await updateWorkOrderPartItemMutation
          .mutateAsync({ id, payload })
          .catch(() => {});
      } else {
        await createWorkOrderPartItemMutation
          .mutateAsync({
            workOrderId: woId!,
            status: nativeForm.data.status as PartStatus,
            code: nativeForm.data.code,
            name: nativeForm.data.name,
            quantity: nativeForm.data.quantity,
            unitType: nativeForm.data.unitType,
            costCents: nativeForm.data.cost * 100,
            markup: nativeForm.data.markup,
          })
          .catch(() => {});
      }
      open = false;
      form.reset();
    },
  });

  const { form: formData, enhance, submitting } = form;

  $effect(() => {
    if (open) {
      if (partItem) {
        $formData.status = partItem.status;
        $formData.code = partItem.code;
        $formData.name = partItem.name;
        $formData.quantity = partItem.quantity;
        $formData.unitType = partItem.unitType;
        $formData.cost = partItem.costCents / 100;
        $formData.markup = partItem.markup;
      } else {
        form.reset();
        $formData.unitType = "штука";
      }
    }
  });
</script>

<MutationSheetShell
  bind:open
  {formID}
  title={isEdit ? "Редагувати запчастину" : "Створити запчастину"}
  description={isEdit
    ? "Змініть дані запчастини"
    : "Внесіть дані нової запчастини"}
  submitLabel={isEdit ? "Зберегти зміни" : "Створити"}
  submitting={$submitting}
  enhanceAction={enhance}
  onCancel={() => {
    form.reset();
    open = false;
  }}
>
  <Form.Field {form} name="status">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Статус</Label>
        <div class="block" {...props}>
          <PartStatusSelector bind:status={$formData.status as PartStatus} />
        </div>
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="code">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Код</Label>
        <Input {...props} bind:value={$formData.code} />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="name">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Назва</Label>
        <Input {...props} bind:value={$formData.name} />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="quantity">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Кількість</Label>
        <Input
          type="number"
          step="0.1"
          min="0"
          {...props}
          bind:value={$formData.quantity}
        />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="unitType">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Одиниці вимірювання</Label>
        <Input {...props} bind:value={$formData.unitType} />
      {/snippet}
    </Form.Control>
    <div class="flex gap-2 items-center">
      <Button
        variant="outline"
        size="sm"
        onclick={() => {
          $formData.unitType = "штука";
        }}>штука</Button
      >
      <Button
        variant="outline"
        size="sm"
        onclick={() => {
          $formData.unitType = "літр";
        }}>літр</Button
      >
    </div>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="cost">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Ціна</Label>
        <div class="flex gap-2 items-baseline" {...props}>
          <CurrencyInput
            bind:value={$formData.cost}
            onkeydown={(e: KeyboardEvent) => {
              if (e.key === "Enter") {
                e.preventDefault();
                const input = e.currentTarget as HTMLInputElement;
                input.blur();
                input.closest("form")?.requestSubmit();
              }
            }}
          />
          <p>грн.</p>
        </div>
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="markup">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Націнка</Label>
        <div class="flex gap-2 items-baseline" {...props}>
          <Input
            type="number"
            min="0"
            step="0.01"
            bind:value={$formData.markup}
          />
          <p>%</p>
        </div>
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>
</MutationSheetShell>

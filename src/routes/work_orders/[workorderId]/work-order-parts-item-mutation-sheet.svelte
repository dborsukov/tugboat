<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";
  import { tick } from "svelte";
  import { NumberFormatStyle, NumericFormat } from "svelte-number-format";
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
  import * as Sheet from "$lib/components/ui/sheet";
  import { Spinner } from "$lib/components/ui/spinner";

  import {
    createWorkOrderPartItemsQuery,
    invalidateWorkOrderLists,
  } from "../queries";
  import PartStatusSelector from "./part-status-selector.svelte";

  interface Props {
    open: boolean;
    id?: number | null;
    woId?: string | null;
  }

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
  const titleText = $derived(
    isEdit ? "Редагувати запчастину" : "Створити запчастину",
  );
  const descText = $derived(
    isEdit
      ? "Змініть дані запчастини тут."
      : "Внесіть дані нової запчастини тут.",
  );
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
      <Form.Field {form} name="status">
        <Form.Control>
          {#snippet children({ props })}
            <Label>Статус</Label>
            <div class="block">
              <PartStatusSelector
                bind:status={$formData.status as PartStatus}
              />
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
              <NumericFormat
                bind:value={$formData.cost}
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

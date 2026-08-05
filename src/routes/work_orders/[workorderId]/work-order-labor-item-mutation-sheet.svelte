<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";
  import { toast } from "svelte-sonner";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import {
    commands,
    type UpdateWorkOrderLaborItemPayload,
  } from "$lib/bindings";

  import { unwrap } from "$lib/misc";

  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";

  import MutationSheetShell from "$lib/components/mutation-sheet-shell.svelte";

  import {
    createCreateWorkOrderLaborItemMutation,
    createWorkOrderLaborItemsQuery,
    invalidateWorkOrderLists,
  } from "../queries";

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

  let laborItemsQuery = createWorkOrderLaborItemsQuery();
  let laborItems = $derived(laborItemsQuery.data ?? []);
  let laborItem = $derived(laborItems.find((c) => c.id === id) ?? null);

  const createItemMutation =
    createCreateWorkOrderLaborItemMutation(queryClient);

  const isEdit = $derived(!!laborItem);
  const formID = $derived(
    isEdit
      ? `work-order-labor-item-mutation-form-${id}`
      : "work-order-labor-item-mutation-form-create",
  );

  const laborItemSchema = z.object({
    code: z.string().nullable(),
    name: z.string().min(1, { error: "Вкажіть назву" }),
    laborHours: z
      .number({ error: "Вкажіть кількість норм-годин" })
      .nonnegative("Неправильна кількість")
      .default(0),
  });

  const updateWorkOrderLaborItemMutation = createMutation(() => ({
    mutationFn: async (variables: {
      id: number;
      payload: UpdateWorkOrderLaborItemPayload;
    }) =>
      unwrap(
        commands.updateWorkOrderLaborItem(variables.id, variables.payload),
      ),
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

  const form = superForm(defaults(zod4(laborItemSchema)), {
    SPA: true,
    validators: zod4Client(laborItemSchema),
    validationMethod: "oninput",
    resetForm: false,
    // svelte-ignore state_referenced_locally
    id: formID,
    async onUpdate({ form: nativeForm }) {
      if (!nativeForm.valid) return;

      if (isEdit && id) {
        await updateWorkOrderLaborItemMutation
          .mutateAsync({
            id,
            payload: {
              serviceId: null,
              code: nativeForm.data.code,
              name: nativeForm.data.name,
              laborHours: nativeForm.data.laborHours,
            },
          })
          .catch(() => {});
      } else {
        await createItemMutation
          .mutateAsync({
            workOrderId: woId!,
            serviceId: null,
            code: nativeForm.data.code,
            name: nativeForm.data.name,
            laborHours: nativeForm.data.laborHours,
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
      if (laborItem) {
        $formData.code = laborItem.code;
        $formData.name = laborItem.name;
        $formData.laborHours = laborItem.laborHours;
      } else {
        form.reset();
      }
    }
  });
</script>

<MutationSheetShell
  bind:open
  {formID}
  title={isEdit ? "Редагувати сервіс" : "Створити сервіс"}
  description={isEdit ? "Змініть дані сервісу" : "Внесіть дані нового сервісу"}
  submitLabel={isEdit ? "Зберегти зміни" : "Створити"}
  submitting={$submitting}
  enhanceAction={enhance}
  onCancel={() => {
    form.reset();
    open = false;
  }}
>
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

  <Form.Field {form} name="laborHours">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Норм-години</Label>
        <Input
          type="number"
          step="0.1"
          min="0"
          {...props}
          bind:value={$formData.laborHours}
        />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>
</MutationSheetShell>

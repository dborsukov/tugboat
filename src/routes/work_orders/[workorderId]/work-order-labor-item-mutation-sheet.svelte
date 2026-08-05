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

  import { Button } from "$lib/components/ui/button";
  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Sheet from "$lib/components/ui/sheet";
  import { Spinner } from "$lib/components/ui/spinner";

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
  const titleText = $derived(isEdit ? "Редагувати сервіс" : "Створити сервіс");
  const descText = $derived(
    isEdit ? "Змініть дані сервіса тут." : "Внесіть дані нового сервіса тут.",
  );
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

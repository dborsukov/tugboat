<script lang="ts">
  import { tick } from "svelte";
  import { toast } from "svelte-sonner";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import { commands, type PartStatus } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import { Button } from "$lib/components/ui/button";
  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Sheet from "$lib/components/ui/sheet";
  import { Spinner } from "$lib/components/ui/spinner";

  import PartStatusSelector from "./part-status-selector.svelte";

  interface Props {
    open: boolean;
    id?: number | null;
    work_order_id?: string | null;
  }

  let {
    open = $bindable(false),
    id = null,
    work_order_id = null,
  }: Props = $props();

  let work_order_item = $derived(
    appStore.work_order_part_items.find((i) => i.id === id) ?? null,
  );

  const isEdit = $derived(!!work_order_item);
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

  const workOrderItemSchema = z.object({
    code: z.string().nullable(),
    name: z.string().min(1, { error: "Вкажіть назву" }),
    quantity: z
      .number({ error: "Вкажіть кількість" })
      .nonnegative("Неправильна кількість")
      .default(1),
    unit_type: z.string().min(1, { error: "Вкажіть одиницю вимірювання" }),
    unit_price_cents: z
      .number({ error: "Вкажіть ціну" })
      .nonnegative("Неправильна ціна")
      .default(0),
    status: z.string().default("Quoted"),
  });

  const form = superForm(defaults(zod4(workOrderItemSchema)), {
    SPA: true,
    validators: zod4Client(workOrderItemSchema),
    validationMethod: "oninput",
    resetForm: false,
    // svelte-ignore state_referenced_locally
    id: formID,
    async onUpdate({ form: nativeForm }) {
      if (!nativeForm.valid) return;

      const payload = {
        code: $formData.code,
        name: $formData.name,
        quantity: $formData.quantity,
        unit_type: $formData.unit_type,
        unit_price_cents: $formData.unit_price_cents,
        status: $formData.status as PartStatus,
      };

      if (isEdit && id) {
        let res = await commands.updateWorkOrderPartItem(id, payload);
        if (res.status === "ok") {
          appStore.updateWorkOrderPartItem(res.data);
        } else {
          return toast.error("Виникла помилка", {
            description: res.error.message,
            position: "bottom-left",
          });
        }
      } else {
        if (!work_order_id) {
          return;
        }
        let res = await commands.createWorkOrderPartItem({
          work_order_id,
          ...payload,
        });
        if (res.status === "ok") {
          appStore.createWorkOrderPartItem(res.data);
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
      if (work_order_item) {
        $formData.code = work_order_item.code;
        $formData.name = work_order_item.name;
        $formData.quantity = work_order_item.quantity;
        $formData.unit_type = work_order_item.unit_type;
        $formData.unit_price_cents = work_order_item.unit_price_cents;
        $formData.status = work_order_item.status;
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
              {...props}
              bind:value={$formData.quantity}
            />
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>

      <Form.Field {form} name="unit_type">
        <Form.Control>
          {#snippet children({ props })}
            <Label>Одиниці вимірювання</Label>
            <Input {...props} bind:value={$formData.unit_type} />
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>

      <Form.Field {form} name="unit_price_cents">
        <Form.Control>
          {#snippet children({ props })}
            <Label>Ціна</Label>
            <Input
              type="number"
              {...props}
              bind:value={$formData.unit_price_cents}
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

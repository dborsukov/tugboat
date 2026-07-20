<script lang="ts">
  import { tick } from "svelte";
  import { toast } from "svelte-sonner";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import { commands } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import { Button } from "$lib/components/ui/button";
  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Sheet from "$lib/components/ui/sheet";
  import { Spinner } from "$lib/components/ui/spinner";

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
    appStore.work_order_labor_items.find((i) => i.id === id) ?? null,
  );

  const isEdit = $derived(!!work_order_item);
  const titleText = $derived(isEdit ? "Редагувати сервіс" : "Створити сервіс");
  const descText = $derived(
    isEdit ? "Змініть дані сервіса тут." : "Внесіть дані нового сервіса тут.",
  );
  const formID = $derived(
    isEdit
      ? `work-order-labor-item-mutation-form-${id}`
      : "work-order-labor-item-mutation-form-create",
  );

  const workOrderItemSchema = z.object({
    code: z.string().nullable(),
    name: z.string().min(1, { error: "Вкажіть назву" }),
    labor_hours: z
      .number({ error: "Вкажіть кількість норм-годин" })
      .nonnegative("Неправильна кількість")
      .default(0),
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
        service_id: null,
        code: nativeForm.data.code,
        name: nativeForm.data.name,
        labor_hours: nativeForm.data.labor_hours,
      };

      if (isEdit && id) {
        let res = await commands.updateWorkOrderLaborItem(id, payload);
        if (res.status === "ok") {
          appStore.updateWorkOrderLaborItem(res.data);
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
        const create_payload = {
          work_order_id: work_order_id,
          ...payload,
        };
        let res = await commands.createWorkOrderLaborItem(create_payload);
        if (res.status === "ok") {
          appStore.createWorkOrderLaborItem(res.data);
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
        $formData.labor_hours = work_order_item.labor_hours;
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

      <Form.Field {form} name="labor_hours">
        <Form.Control>
          {#snippet children({ props })}
            <Label>Норм-години</Label>
            <Input
              type="number"
              step="0.1"
              {...props}
              bind:value={$formData.labor_hours}
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

<script lang="ts">
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
  }

  let { open = $bindable(false), id = null }: Props = $props();

  let service = $derived(appStore.services.find((s) => s.id === id) ?? null);

  const isEdit = $derived(!!id);
  const titleText = $derived(isEdit ? "Редагувати сервіс" : "Додати сервіс");
  const descText = $derived(
    isEdit
      ? "Змініть параметри сервісу тут."
      : "Внесіть дані нового сервісу тут.",
  );
  const formID = $derived(
    isEdit ? `service-mutation-form-${id}` : "service-mutation-form-create",
  );

  export const serviceSchema = z.object({
    name: z.string().nonempty("Назва не може бути пустою"),
    unit_price: z
      .number({ error: "Вкажіть ціну" })
      .nonnegative("Неправильна ціна")
      .default(0),
  });

  const form = superForm(defaults(zod4(serviceSchema)), {
    SPA: true,
    validators: zod4Client(serviceSchema),
    validationMethod: "oninput",
    resetForm: false,
    // svelte-ignore state_referenced_locally
    id: formID,
    async onUpdate({ form: nativeForm }) {
      if (!nativeForm.valid) return;

      const payload = {
        name: $formData.name,
        unit_price: $formData.unit_price,
      };

      try {
        if (isEdit && id) {
          let res = await commands.updateService(id, payload);
          if (res.status === "ok") {
            appStore.updateService(res.data);
          } else {
            // TODO: show error to user
            alert(res.error);
          }
        } else {
          let res = await commands.createService(payload);
          if (res.status === "ok") {
            appStore.createService(res.data);
          } else {
            // TODO: show error to user
            alert(res.error);
          }
        }
        open = false;
        form.reset();
      } catch (err) {
        alert(`Critical IPC error: ${err}`);
      }
    },
  });

  const { form: formData, errors, enhance, submitting } = form;

  $effect(() => {
    if (open) {
      if (service) {
        $formData.name = service.name;
        $formData.unit_price = service.unit_price;
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
      <Form.Field {form} name="name">
        <Form.Control>
          {#snippet children({ props })}
            <Label>Назва</Label>
            <Input {...props} bind:value={$formData.name} />
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>
      <Form.Field {form} name="unit_price">
        <Form.Control>
          {#snippet children({ props })}
            <Label>Ціна</Label>
            <Input {...props} type="number" bind:value={$formData.unit_price} />
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>
    </form>
    <Sheet.Footer>
      <Button type="submit" disabled={$submitting} form={formID}>
        {#if $submitting}<Spinner />{/if}
        {isEdit ? "Зберегти зміни" : "Додати"}
      </Button>
      <Button variant="outline" disabled={$submitting} onclick={handleCancel}>
        Відмінити та закрити
      </Button>
    </Sheet.Footer>
  </Sheet.Content>
</Sheet.Root>

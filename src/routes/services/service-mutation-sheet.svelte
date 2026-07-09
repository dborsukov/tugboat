<script lang="ts">
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
    code: z.string().nullable(),
    name: z.string().min(1, { error: "Вкажіть назву" }),
    labor_hours: z
      .number({ error: "Вкажіть кількість норм-годин" })
      .nonnegative("Неправильна кількість")
      .default(0),
  });

  const extendedSchema = serviceSchema
    .refine(
      (data) => {
        if (data.code == null) return true;
        const servicesList = appStore.services || [];
        const duplicateService = servicesList.find(
          (c) =>
            c.code?.trim().toLowerCase() === data.code?.trim().toLowerCase(),
        );
        if (!duplicateService) return true;
        // Avoid record matching with itself (when editing)
        if (isEdit && duplicateService.id === service?.id) return true;
        return false;
      },
      {
        message: "Сервіс з таким кодом вже існує",
        path: ["code"],
      },
    )
    .refine(
      (data) => {
        const servicesList = appStore.services || [];
        const duplicateService = servicesList.find(
          (c) => c.name.toLowerCase() === data.name.toLowerCase(),
        );
        if (!duplicateService) return true;
        // Avoid record matching with itself (when editing)
        if (isEdit && duplicateService.id === service?.id) return true;
        return false;
      },
      {
        message: "Сервіс з такою назвою вже існує",
        path: ["name"],
      },
    );

  const form = superForm(defaults(zod4(extendedSchema)), {
    SPA: true,
    validators: zod4Client(extendedSchema),
    validationMethod: "oninput",
    resetForm: false,
    // svelte-ignore state_referenced_locally
    id: formID,
    async onUpdate({ form: nativeForm }) {
      if (!nativeForm.valid) return;

      const payload = {
        code: $formData.code,
        name: $formData.name,
        labor_hours: $formData.labor_hours,
      };

      if (isEdit && id) {
        let res = await commands.updateService(id, payload);
        if (res.status === "ok") {
          appStore.updateService(res.data);
        } else {
          return toast.error("Виникла помилка", {
            description: res.error.message,
            position: "bottom-left",
          });
        }
      } else {
        let res = await commands.createService(payload);
        if (res.status === "ok") {
          appStore.createService(res.data);
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
      if (service) {
        $formData.code = service.code;
        $formData.name = service.name;
        $formData.labor_hours = service.labor_hours;
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
              {...props}
              type="number"
              step="0.1"
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
        {isEdit ? "Зберегти зміни" : "Додати"}
      </Button>
      <Button variant="outline" disabled={$submitting} onclick={handleCancel}>
        Відмінити та закрити
      </Button>
    </Sheet.Footer>
  </Sheet.Content>
</Sheet.Root>

<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";
  import { toast } from "svelte-sonner";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import { commands } from "$lib/bindings";

  import { unwrap } from "$lib/misc";

  import { Button } from "$lib/components/ui/button";
  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Sheet from "$lib/components/ui/sheet";
  import { Spinner } from "$lib/components/ui/spinner";

  import { createServicesQuery, invalidateServiceLists } from "./queries";

  let {
    open = $bindable(false),
    id = null,
  }: { open: boolean; id?: number | null } = $props();

  const queryClient = useQueryClient();

  let servicesQuery = createServicesQuery();
  let servicesList = $derived(servicesQuery.data ?? []);
  let service = $derived(servicesList.find((s) => s.id === id) ?? null);

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
    laborHours: z
      .number({ error: "Вкажіть кількість норм-годин" })
      .nonnegative("Неправильна кількість")
      .default(0),
  });

  const extendedSchema = serviceSchema
    .refine(
      (data) => {
        if (data.code == null) return true;
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

  const createServiceMutation = createMutation(() => ({
    mutationFn: async (payload: z.infer<typeof serviceSchema>) =>
      unwrap(commands.createService(payload)),
    onSuccess: () => {
      invalidateServiceLists(queryClient);
      open = false;
      form.reset();
    },
    onError: (error) => {
      toast.error("Виникла помилка", {
        description: error.message,
        position: "bottom-left",
      });
    },
  }));

  const updateServiceMutation = createMutation(() => ({
    mutationFn: async (variables: {
      id: number;
      payload: z.infer<typeof serviceSchema>;
    }) => unwrap(commands.updateService(variables.id, variables.payload)),
    onSuccess: () => {
      invalidateServiceLists(queryClient);
      open = false;
      form.reset();
    },
    onError: (error) => {
      toast.error("Виникла помилка", {
        description: error.message,
        position: "bottom-left",
      });
    },
  }));

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
        code: nativeForm.data.code,
        name: nativeForm.data.name,
        laborHours: nativeForm.data.laborHours,
      };

      if (isEdit && id) {
        await updateServiceMutation
          .mutateAsync({ id, payload })
          .catch(() => {});
      } else {
        await createServiceMutation.mutateAsync(payload).catch(() => {});
      }
      open = false;
      form.reset();
    },
  });

  const { form: formData, enhance, submitting } = form;

  $effect(() => {
    if (open) {
      if (service) {
        form.reset({
          data: {
            code: service.code,
            name: service.name,
            laborHours: service.laborHours,
          },
        });
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
            <Label>Нормо-години</Label>
            <Input
              {...props}
              type="number"
              step="0.1"
              min="0"
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
        {isEdit ? "Зберегти зміни" : "Додати"}
      </Button>
      <Button variant="outline" disabled={$submitting} onclick={handleCancel}>
        Відмінити та закрити
      </Button>
    </Sheet.Footer>
  </Sheet.Content>
</Sheet.Root>

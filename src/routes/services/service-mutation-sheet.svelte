<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";
  import { toast } from "svelte-sonner";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import { commands } from "$lib/bindings";

  import { unwrap } from "$lib/misc";

  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";

  import MutationSheetShell from "$lib/components/mutation-sheet-shell.svelte";

  import { createServicesQuery, invalidateServiceLists } from "./queries";

  let {
    id = null,
    open = $bindable(false),
  }: { id?: number | null; open: boolean } = $props();

  const queryClient = useQueryClient();

  let servicesQuery = createServicesQuery();
  let servicesList = $derived(servicesQuery.data ?? []);
  let service = $derived(servicesList.find((s) => s.id === id) ?? null);

  const isEdit = $derived(!!id);
  const formID = $derived(`service-mutation-form-${id}`);

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
</script>

<MutationSheetShell
  bind:open
  {formID}
  title={isEdit ? "Редагувати сервіс" : "Створити сервіс"}
  description={isEdit ? "Змініть дані сервіса" : "Внесіть дані нового сервіса"}
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
</MutationSheetShell>

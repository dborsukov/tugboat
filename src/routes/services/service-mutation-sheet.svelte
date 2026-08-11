<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import Big from "big.js";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";

  import MutationSheetShell from "$lib/components/mutation-sheet-shell.svelte";

  import {
    createCreateServiceMutation,
    createServicesQuery,
    createUpdateServiceMutation,
  } from "./queries";

  let {
    serviceId = null,
    open = $bindable(false),
  }: { serviceId?: number | null; open: boolean } = $props();

  const formID = "service-mutation-form";
  const isEdit = $derived(serviceId != null);

  const queryClient = useQueryClient();

  const createServiceMutation = createCreateServiceMutation(queryClient);
  const updateServiceMutation = createUpdateServiceMutation(queryClient);

  let servicesQuery = createServicesQuery();
  let servicesList = $derived(servicesQuery.data ?? []);
  let service = $derived(servicesList.find((s) => s.id === serviceId) ?? null);

  export const zBig = z
    .unknown()
    .transform((val, ctx) => {
      if (val === "" || val === null || val === undefined) {
        ctx.addIssue({
          code: "custom",
          message: "Вкажіть кількість норм-годин",
        });
        return z.NEVER;
      }

      if (
        typeof val === "string" ||
        typeof val === "number" ||
        val instanceof Big
      ) {
        try {
          return val instanceof Big ? val : Big(val);
        } catch {
          ctx.addIssue({
            code: "custom",
            message: "Некоректне числове значення",
          });
          return z.NEVER;
        }
      }

      ctx.addIssue({
        code: "custom",
        message: "Вкажіть кількість норм-годин",
      });
      return z.NEVER;
    })
    .refine((val) => val.gte(0), {
      message: "Неправильна кількість",
    });

  export const serviceSchema = z.object({
    code: z.string().nullable(),
    name: z.string().min(1, { error: "Вкажіть назву" }),
    laborHours: zBig,
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
        laborHours: nativeForm.data.laborHours.toString(),
      };

      if (isEdit && serviceId) {
        await updateServiceMutation
          .mutateAsync({ id: serviceId, payload })
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
            laborHours: Big(service.laborHours),
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

<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";

  import MutationSheetShell from "$lib/components/mutation-sheet-shell.svelte";

  import {
    laborPriceFields,
    laborPriceFromEntity,
    laborPriceToPayload,
    refineLaborPrice,
  } from "$lib/schemas/zlaborprice";

  import LaborPriceField from "../work_orders/[workorderId]/labor-price-field.svelte";
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

  export const serviceSchema = z.object({
    code: z.string().nullable(),
    name: z.string().min(1, { error: "Вкажіть назву" }),
    ...laborPriceFields,
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
    )
    .superRefine(refineLaborPrice);

  const form = superForm(defaults(zod4(extendedSchema)), {
    SPA: true,
    validators: zod4Client(extendedSchema),
    validationMethod: "oninput",
    resetForm: false,
    // svelte-ignore state_referenced_locally
    id: formID,
    async onUpdate({ form: nativeForm, cancel }) {
      if (!nativeForm.valid) return;

      const laborPricePayload = laborPriceToPayload(
        nativeForm.data.laborPriceType,
        nativeForm.data.laborPrice,
      );

      const payload = {
        code: nativeForm.data.code,
        name: nativeForm.data.name,
        ...laborPricePayload,
      };

      if (isEdit && serviceId) {
        await updateServiceMutation
          .mutateAsync({ id: serviceId, payload })
          .catch(() => {});
      } else {
        await createServiceMutation.mutateAsync(payload).catch(() => {});
      }
      // Prevent superforms from re-applying the submitted data to the form
      // after this handler, which would overwrite the reset below and leave
      // stale values in the fields for the next time the sheet opens.
      cancel();
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
            laborPriceType: service.priceType,
            laborPrice: laborPriceFromEntity(service),
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

  <Form.Field {form} name="laborPrice">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Вартість</Label>
        <LaborPriceField
          bind:type={$formData.laborPriceType}
          bind:value={$formData.laborPrice}
          inputProps={props}
        />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>
</MutationSheetShell>

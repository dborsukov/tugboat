<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import { tick } from "svelte";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";

  import MutationSheetShell from "$lib/components/mutation-sheet-shell.svelte";
  import { sheetInput, sheetLabel } from "$lib/components/sheet/field-styles";
  import RequiredMark from "$lib/components/sheet/required-mark.svelte";

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
  }: {
    serviceId?: number | null;
    open: boolean;
  } = $props();

  const formID = "service-mutation-form";
  const isEdit = $derived(serviceId != null);

  const queryClient = useQueryClient();
  const createServiceMutation = createCreateServiceMutation(queryClient);
  const updateServiceMutation = createUpdateServiceMutation(queryClient);

  const servicesQuery = createServicesQuery();
  const servicesList = $derived(servicesQuery.data ?? []);
  const service = $derived(
    servicesList.find((s) => s.id === serviceId) ?? null,
  );

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
      { message: "Сервіс з таким кодом вже існує", path: ["code"] },
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
      { message: "Сервіс з такою назвою вже існує", path: ["name"] },
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
      submitError = null;
      try {
        if (isEdit && serviceId) {
          await updateServiceMutation.mutateAsync({ id: serviceId, payload });
        } else {
          await createServiceMutation.mutateAsync(payload);
        }
      } catch (error) {
        submitError =
          error instanceof Error ? error.message : "Не вдалося зберегти";
        return;
      }
      cancel();
      open = false;
      form.reset();
      baseline = null;
      loadedKey = null;
    },
  });

  const { form: formData, enhance, submitting } = form;

  let loadedKey = $state<string | null>(null);
  let baseline = $state<string | null>(null);
  let submitError = $state<string | null>(null);
  const dirty = $derived(
    baseline !== null && JSON.stringify($formData) !== baseline,
  );

  function markBaseline() {
    baseline = JSON.stringify($formData);
  }

  $effect(() => {
    if (!open) {
      loadedKey = null;
      baseline = null;
      submitError = null;
      return;
    }
    const key = isEdit ? String(serviceId) : "new";
    if (loadedKey === key) return;
    if (isEdit && !service) return;
    loadedKey = key;

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

    void tick().then(() => {
      if (!open || loadedKey !== key) return;
      markBaseline();
      setTimeout(() => {
        if (open && loadedKey === key) markBaseline();
      }, 150);
    });
  });
</script>

<MutationSheetShell
  bind:open
  {formID}
  {dirty}
  bodyClass="gap-0"
  title={isEdit ? "Редагувати сервіс" : "Створити сервіс"}
  description={isEdit ? "Змініть дані сервісу" : "Внесіть дані нового сервісу"}
  submitLabel={isEdit ? "Зберегти зміни" : "Створити"}
  submitting={$submitting}
  error={submitError}
  enhanceAction={enhance}
  onCancel={() => {
    form.reset();
    baseline = null;
    loadedKey = null;
    submitError = null;
    open = false;
  }}
>
  <div class="flex flex-col gap-4">
    <div class="flex flex-col gap-4">
      <div class="grid grid-cols-[minmax(0,0.8fr)_minmax(0,1.2fr)] gap-3">
        <Form.Field {form} name="code" class="space-y-1.5">
          <Form.Control>
            {#snippet children({ props })}
              <Label class={sheetLabel}>Код</Label>
              <Input
                {...props}
                data-sheet-autofocus
                bind:value={$formData.code}
                placeholder="SRV-001"
                class={sheetInput}
              />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>

        <Form.Field {form} name="name" class="space-y-1.5">
          <Form.Control>
            {#snippet children({ props })}
              <Label class={sheetLabel}>Назва{@render requiredMark()}</Label>
              <Input
                {...props}
                bind:value={$formData.name}
                placeholder="Наприклад: Заміна масла"
                class={sheetInput}
              />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>
      </div>
    </div>

    <Form.Field {form} name="laborPrice" class="space-y-1.5">
      <Form.Control>
        {#snippet children({ props })}
          <Label class={sheetLabel}>Вартість{@render requiredMark()}</Label>
          <LaborPriceField
            bind:type={$formData.laborPriceType}
            bind:value={$formData.laborPrice}
            inputProps={props}
          />
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>
  </div>
</MutationSheetShell>

{#snippet requiredMark()}
  <RequiredMark />
{/snippet}

<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import type { CountryCode } from "svelte-tel-input/types";
  import { validateTelInput } from "svelte-tel-input/validators";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Textarea } from "$lib/components/ui/textarea";

  import MutationSheetShell from "$lib/components/mutation-sheet-shell.svelte";
  import PhoneInput from "$lib/components/phone-input.svelte";

  import {
    createClientsQuery,
    createCreateClientMutation,
    createUpdateClientMutation,
  } from "./queries";

  let {
    clientId = null,
    open = $bindable(false),
  }: { clientId?: number | null; open: boolean } = $props();

  const formID = "client-mutation-form";
  const isEdit = $derived(clientId != null);

  const queryClient = useQueryClient();

  const createClientMutation = createCreateClientMutation(queryClient);
  const updateClientMutation = createUpdateClientMutation(queryClient);

  let clientsQuery = createClientsQuery();
  let clientsList = $derived(clientsQuery.data ?? []);
  let client = $derived(clientsList.find((c) => c.id === clientId) ?? null);

  let phoneCountry = $state<CountryCode>("UA");

  export const clientSchema = z.object({
    name: z.string().nonempty("Обов'язкове поле"),
    phone: z.string().superRefine((val, ctx) => {
      const error = validateTelInput(val, { required: true });
      if (error) {
        ctx.addIssue({
          code: "custom",
          message:
            error === "REQUIRED" ? "Обов'язкове поле" : "Неправильний формат",
        });
      }
    }),
    notes: z
      .string()
      .nullable()
      .default(null)
      .transform((val) => (val === "" ? null : val)),
  });

  const extendedSchema = clientSchema.refine(
    (data) => {
      const duplicateClient = clientsList.find((c) => c.phone === data.phone);
      if (!duplicateClient) return true;
      if (isEdit && duplicateClient.id === client?.id) return true;
      return false;
    },
    {
      message: "Клієнт з таким номером вже існує",
      path: ["phone"],
    },
  );

  const form = superForm(defaults(zod4(extendedSchema)), {
    SPA: true,
    validators: zod4Client(extendedSchema),
    validationMethod: "oninput",
    resetForm: false,
    id: formID,
    async onUpdate({ form: nativeForm }) {
      if (!nativeForm.valid) return;

      const payload = {
        name: nativeForm.data.name,
        phone: nativeForm.data.phone,
        notes: nativeForm.data.notes?.trim() ?? null,
      };

      if (isEdit && clientId) {
        await updateClientMutation
          .mutateAsync({ id: clientId, payload })
          .catch(() => {});
      } else {
        await createClientMutation.mutateAsync(payload).catch(() => {});
      }
      open = false;
      form.reset();
    },
  });

  const { form: formData, enhance, submitting } = form;

  $effect(() => {
    if (open) {
      if (client) {
        $formData.name = client.name;
        $formData.phone = client.phone ?? "";
        $formData.notes = client.notes;
      } else {
        form.reset();
      }
    }
  });
</script>

{#snippet requiredHint()}
  <span
    class="text-destructive inline-flex align-middle"
    title="Обов'язкове поле"
    aria-label="Обов'язкове поле"
  >
    *
  </span>
{/snippet}

<MutationSheetShell
  bind:open
  {formID}
  title={isEdit ? "Редагувати клієнта" : "Створити клієнта"}
  description={isEdit ? "Змініть дані клієнта" : "Внесіть дані нового клієнта"}
  submitLabel={isEdit ? "Зберегти зміни" : "Створити"}
  submitting={$submitting}
  enhanceAction={enhance}
  onCancel={() => {
    form.reset();
    open = false;
  }}
>
  <Form.Field {form} name="name">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Ім'я {@render requiredHint()}</Label>
        <Input {...props} bind:value={$formData.name} />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="phone">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Мобільний номер{@render requiredHint()}</Label>
        <PhoneInput
          {...props}
          bind:value={$formData.phone}
          bind:country={phoneCountry}
        />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="notes">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Нотатки</Label>
        <Textarea
          {...props}
          value={$formData.notes ?? ""}
          oninput={(e) => {
            $formData.notes = e.currentTarget.value;
          }}
          placeholder="Додаткова інформація про клієнта..."
          rows={3}
        />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>
</MutationSheetShell>

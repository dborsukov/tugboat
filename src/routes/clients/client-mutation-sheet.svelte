<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";
  import { toast } from "svelte-sonner";
  import type { CountryCode } from "svelte-tel-input/types";
  import { validateTelInput } from "svelte-tel-input/validators";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";

  import MutationSheetShell from "$lib/components/mutation-sheet-shell.svelte";
  import PhoneInput from "$lib/components/phone-input.svelte";

  import { commands } from "$lib/bindings";
  import { unwrap } from "$lib/tools";

  import { createClientsQuery, invalidateClientsLists } from "./queries";

  let {
    id = null,
    open = $bindable(false),
  }: { id?: number | null; open: boolean } = $props();

  const queryClient = useQueryClient();

  let clientsQuery = createClientsQuery();
  let clientsList = $derived(clientsQuery.data ?? []);
  let client = $derived(clientsList.find((c) => c.id === id) ?? null);

  const isEdit = $derived(!!client);
  const formID = "client-mutation-form";

  let phoneCountry = $state<CountryCode>("UA");

  export const clientSchema = z.object({
    name: z.string().nonempty("Ім'я не може бути пустим"),
    phone: z.string().superRefine((val, ctx) => {
      const error = validateTelInput(val, {
        required: true,
        allowedCountries: ["UA"],
      });
      if (error) {
        ctx.addIssue({
          code: "custom",
          message: "Введіть коректний номер телефону",
        });
      }
    }),
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

  const createClientMutation = createMutation(() => ({
    mutationFn: async (payload: z.infer<typeof clientSchema>) =>
      unwrap(commands.createClient(payload)),
    onSuccess: () => {
      invalidateClientsLists(queryClient);
    },
    onError: (error) => {
      toast.error("Виникла помилка", {
        description: error.message,
        position: "bottom-left",
      });
    },
  }));

  const updateClientMutation = createMutation(() => ({
    mutationFn: async (variables: {
      id: number;
      payload: z.infer<typeof clientSchema>;
    }) => unwrap(commands.updateClient(variables.id, variables.payload)),
    onSuccess: () => {
      invalidateClientsLists(queryClient);
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
    id: formID,
    async onUpdate({ form: nativeForm }) {
      if (!nativeForm.valid) return;

      const payload = {
        name: nativeForm.data.name,
        phone: nativeForm.data.phone,
      };

      if (isEdit && id) {
        await updateClientMutation.mutateAsync({ id, payload }).catch(() => {});
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
      } else {
        form.reset();
      }
    }
  });
</script>

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
        <Label>Ім'я</Label>
        <Input {...props} bind:value={$formData.name} />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="phone">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Мобільний номер</Label>
        <PhoneInput
          {...props}
          bind:value={$formData.phone}
          bind:country={phoneCountry}
          options={{ lockCountry: true, spaces: true }}
        />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>
</MutationSheetShell>

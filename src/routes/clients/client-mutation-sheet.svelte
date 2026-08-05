<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";
  import { toast } from "svelte-sonner";
  import { TelInput } from "svelte-tel-input";
  import type { CountryCode } from "svelte-tel-input/types";
  import { validateTelInput } from "svelte-tel-input/validators";
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
  const titleText = $derived(
    isEdit ? "Редагувати клієнта" : "Створити клієнта",
  );
  const descText = $derived(
    isEdit ? "Змініть дані клієнта тут." : "Внесіть дані нового клієнта тут.",
  );
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
      // Avoid record matching with itself (when editing)
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
            <TelInput
              {...props}
              bind:value={$formData.phone}
              bind:country={phoneCountry}
              options={{ lockCountry: true, spaces: true }}
              placeholder="+380 XX XXX XX XX"
              class="bg-input/50 focus-visible:border-ring focus-visible:ring-ring/30 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive dark:aria-invalid:border-destructive/50 h-8 rounded-2xl border border-transparent px-2.5 py-1 text-base transition-[color,box-shadow] duration-200 file:h-6 file:text-sm file:font-medium focus-visible:ring-3 aria-invalid:ring-3 md:text-sm file:text-foreground placeholder:text-muted-foreground w-full min-w-0 outline-none file:inline-flex file:border-0 file:bg-transparent disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50"
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

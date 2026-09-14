<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import { Checkbox } from "$lib/components/ui/checkbox";
  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";

  import MutationSheetShell from "$lib/components/mutation-sheet-shell.svelte";

  import {
    createCreateWorkerMutation,
    createUpdateWorkerMutation,
    createWorkersQuery,
  } from "./queries";

  let {
    workerId = null,
    open = $bindable(false),
  }: { workerId?: number | null; open: boolean } = $props();

  const formID = "worker-mutation-form";
  const isEdit = $derived(workerId != null);

  const queryClient = useQueryClient();

  const createWorkerMutation = createCreateWorkerMutation(queryClient);
  const updateWorkerMutation = createUpdateWorkerMutation(queryClient);

  const workersQuery = createWorkersQuery();
  const workersList = $derived(workersQuery.data ?? []);
  const worker = $derived(workersList.find((w) => w.id === workerId) ?? null);

  export const workerSchema = z.object({
    firstName: z.string().min(1, { error: "Вкажіть ім'я" }),
    surname: z.string().min(1, { error: "Вкажіть прізвище" }),
    patronymic: z.string().default(""),
    position: z.string().default(""),
    isActive: z.boolean().default(true),
  });

  const form = superForm(defaults(zod4(workerSchema)), {
    SPA: true,
    validators: zod4Client(workerSchema),
    validationMethod: "oninput",
    resetForm: false,
    // svelte-ignore state_referenced_locally
    id: formID,
    async onUpdate({ form: nativeForm, cancel }) {
      if (!nativeForm.valid) return;

      const payload = {
        firstName: nativeForm.data.firstName.trim(),
        surname: nativeForm.data.surname.trim(),
        patronymic: nativeForm.data.patronymic.trim() || null,
        position: nativeForm.data.position.trim() || null,
        isActive: nativeForm.data.isActive,
      };

      if (isEdit && workerId) {
        await updateWorkerMutation
          .mutateAsync({ id: workerId, payload })
          .catch(() => {});
      } else {
        await createWorkerMutation.mutateAsync(payload).catch(() => {});
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
      if (worker) {
        form.reset({
          data: {
            firstName: worker.firstName,
            surname: worker.surname,
            patronymic: worker.patronymic ?? "",
            position: worker.position ?? "",
            isActive: worker.isActive,
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
  title={isEdit ? "Редагувати працівника" : "Створити працівника"}
  description={isEdit
    ? "Змініть дані працівника"
    : "Внесіть дані нового працівника"}
  submitLabel={isEdit ? "Зберегти зміни" : "Створити"}
  submitting={$submitting}
  enhanceAction={enhance}
  onCancel={() => {
    form.reset();
    open = false;
  }}
>
  <Form.Field {form} name="surname">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Прізвище</Label>
        <Input {...props} bind:value={$formData.surname} />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="firstName">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Ім'я</Label>
        <Input {...props} bind:value={$formData.firstName} />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="patronymic">
    <Form.Control>
      {#snippet children({ props })}
        <Label>По-батькові</Label>
        <Input {...props} bind:value={$formData.patronymic} />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="position">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Посада</Label>
        <Input
          {...props}
          bind:value={$formData.position}
          placeholder="Наприклад: автослюсар"
        />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="isActive">
    <Form.Control>
      {#snippet children({ props })}
        <div class="flex items-center gap-2" {...props}>
          <Checkbox
            id="worker-is-active"
            checked={$formData.isActive}
            onCheckedChange={(checked) =>
              ($formData.isActive = checked === true)}
          />
          <Label for="worker-is-active" class="font-normal">
            Активний працівник
          </Label>
        </div>
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>
</MutationSheetShell>

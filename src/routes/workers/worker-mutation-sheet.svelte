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
  import SheetSection from "$lib/components/sheet/sheet-section.svelte";
  import SheetSwitchRow from "$lib/components/sheet/sheet-switch-row.svelte";

  import {
    createCreateWorkerMutation,
    createUpdateWorkerMutation,
    createWorkersQuery,
  } from "./queries";

  let {
    workerId = null,
    open = $bindable(false),
  }: {
    workerId?: number | null;
    open: boolean;
  } = $props();

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
      return;
    }
    const key = isEdit ? String(workerId) : "new";
    if (loadedKey === key) return;
    if (isEdit && !worker) return;
    loadedKey = key;

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
  title={isEdit ? "Редагувати працівника" : "Створити працівника"}
  description={isEdit
    ? "Змініть дані працівника"
    : "Внесіть дані нового працівника"}
  submitLabel={isEdit ? "Зберегти зміни" : "Створити"}
  submitting={$submitting}
  enhanceAction={enhance}
  onCancel={() => {
    form.reset();
    baseline = null;
    loadedKey = null;
    open = false;
  }}
>
  <SheetSection title="Основне" class="pt-0">
    <div class="flex flex-col gap-4">
      <div class="grid grid-cols-3 gap-3">
        <Form.Field {form} name="surname" class="space-y-1.5">
          <Form.Control>
            {#snippet children({ props })}
              <Label class={sheetLabel}>
                Прізвище{@render requiredMark()}
              </Label>
              <Input
                {...props}
                data-sheet-autofocus
                bind:value={$formData.surname}
                placeholder="Іванов"
                class={sheetInput}
              />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>

        <Form.Field {form} name="firstName" class="space-y-1.5">
          <Form.Control>
            {#snippet children({ props })}
              <Label class={sheetLabel}>Ім'я{@render requiredMark()}</Label>
              <Input
                {...props}
                bind:value={$formData.firstName}
                placeholder="Іван"
                class={sheetInput}
              />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>

        <Form.Field {form} name="patronymic" class="space-y-1.5">
          <Form.Control>
            {#snippet children({ props })}
              <Label class={sheetLabel}>По-батькові</Label>
              <Input
                {...props}
                bind:value={$formData.patronymic}
                placeholder="Іванович"
                class={sheetInput}
              />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>
      </div>

      <Form.Field {form} name="position" class="space-y-1.5">
        <Form.Control>
          {#snippet children({ props })}
            <Label class={sheetLabel}>Посада</Label>
            <Input
              {...props}
              bind:value={$formData.position}
              placeholder="Наприклад: автослюсар"
              class={sheetInput}
            />
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>
    </div>
  </SheetSection>

  <SheetSection title="Доступ">
    <Form.Field {form} name="isActive" class="space-y-1.5">
      <Form.Control>
        {#snippet children({ props })}
          <SheetSwitchRow
            id="worker-is-active"
            label="Активний працівник"
            checked={$formData.isActive}
            onCheckedChange={(checked) => ($formData.isActive = checked)}
            switchProps={props}
          >
            Працівник доступний у виборі виконавця.
          </SheetSwitchRow>
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>
  </SheetSection>
</MutationSheetShell>

{#snippet requiredMark()}
  <RequiredMark />
{/snippet}

<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import { toast } from "svelte-sonner";

  import PenLineIcon from "@lucide/svelte/icons/pen-line";

  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import Spinner from "$lib/components/ui/spinner/spinner.svelte";
  import { Textarea } from "$lib/components/ui/textarea";

  import AppTooltip from "$lib/components/app-tooltip.svelte";
  import ComboSelect from "$lib/components/combo-select.svelte";
  import DateTimePicker from "$lib/components/date-time-picker.svelte";

  import {
    type ClientType,
    type DocumentKind,
    type GenerateDocumentOptions,
    type WorkOrder,
    type WorkOrderDetailsDto,
  } from "$lib/bindings";
  import { pibInitials } from "$lib/names";
  import { settingsState } from "$lib/states/settings.svelte";

  import { createClientDetailsQuery } from "../../clients/queries";
  import { workerFullName, workerShortName } from "../../workers/labels";
  import { createWorkersQuery } from "../../workers/queries";
  import WorkerOption from "../../workers/worker-option.svelte";
  import {
    createGenerateDocumentMutation,
    createOpenWorkOrderDocumentMutation,
  } from "../queries";

  let {
    workOrderId,
    workOrderDetails,
    kind,
    /** Choices of the version the dialog was opened from, if it's an update. */
    seed = null,
    open = $bindable(false),
  }: {
    workOrderId: string;
    workOrderDetails: WorkOrderDetailsDto | undefined;
    kind: DocumentKind;
    seed?: GenerateDocumentOptions | null;
    open: boolean;
  } = $props();

  const MANUAL_TOGGLE_TOOLTIP = "Пропустити та заповнити від руки пізніше";

  const queryClient = useQueryClient();
  const generateDocumentMutation = createGenerateDocumentMutation(queryClient);
  const openDocumentMutation = createOpenWorkOrderDocumentMutation(queryClient);

  const workersQuery = createWorkersQuery();

  const customerClientQuery = createClientDetailsQuery(
    () => workOrderDetails?.workOrder.customerClientId ?? null,
  );

  const isNaryad = $derived(kind === "naryad_zamovlennya");
  const isAkt = $derived(kind === "akt_vykonanykh_robit");

  // ---------- Form state ----------

  let performerWorkerId = $state<number | null>(null);
  let skipPerformerSignature = $state(false);
  let skipCustomerSignature = $state(false);
  let receivedAt = $state("");
  let plannedCompletionAt = $state("");
  let skipDate = $state(false);
  let location = $state("");
  let completedAt = $state("");
  let skipLocation = $state(false);
  let odometerOut = $state("");
  let additionalRecommendations = $state("");

  let error = $state<string | null>(null);

  let generating = $state(false);

  const performerWorker = $derived(
    (workersQuery.data ?? []).find((w) => w.id === performerWorkerId) ?? null,
  );

  const performerSignatureName = $derived(
    skipPerformerSignature || !performerWorker
      ? ""
      : workerShortName(performerWorker),
  );

  const customerSignatureName = $derived(
    skipCustomerSignature
      ? ""
      : resolveCustomerSignatureName(
          customerClientQuery.data?.client ?? workOrderDetails?.client,
          workOrderDetails?.workOrder,
        ),
  );

  function pad(n: number): string {
    return String(n).padStart(2, "0");
  }

  function toLocalDateTimeValue(iso: string | null | undefined): string {
    if (!iso) return "";
    const d = new Date(iso);
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
  }

  function toLocalDateValue(iso: string | null | undefined): string {
    if (!iso) return "";
    const d = new Date(iso);
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
  }

  type SignatureSource = {
    name: string;
    surname: string | null;
    firstName: string | null;
    patronymic: string | null;
    clientType: ClientType;
  };

  // Mirrors: src-tauri/src/documents/model.rs
  function resolveCustomerSignatureName(
    client: SignatureSource | undefined,
    workOrder: WorkOrder | undefined,
  ): string {
    if (!client) return "";
    if (client.clientType === "individual") {
      return (
        pibInitials(client.surname, client.firstName, client.patronymic) ||
        client.name
      );
    }
    return pibInitials(
      workOrder?.driverSurname,
      workOrder?.driverFirstName,
      workOrder?.driverPatronymic,
    );
  }

  $effect(() => {
    if (open) {
      error = null;
      generating = false;

      const wo = workOrderDetails?.workOrder;
      const previous = seed;
      performerWorkerId =
        previous?.performerWorkerId ??
        workOrderDetails?.worker?.id ??
        wo?.workerId ??
        null;
      skipPerformerSignature = previous?.skipPerformerSignature ?? false;
      skipCustomerSignature = previous?.skipCustomerSignature ?? false;
      receivedAt = toLocalDateTimeValue(
        previous ? previous.receivedAt : wo?.createdAt,
      );
      plannedCompletionAt = toLocalDateTimeValue(
        previous ? previous.plannedCompletionAt : plannedDefaultIso(),
      );
      skipDate = previous?.skipDate ?? false;
      location =
        previous?.location ??
        settingsState.settings?.companyInfo?.physicalAddress ??
        "";
      completedAt = toLocalDateValue(
        previous ? previous.completedAt : new Date().toISOString(),
      );
      skipLocation = previous?.skipLocation ?? false;
      odometerOut =
        previous?.odometerOut != null
          ? String(previous.odometerOut)
          : String(wo?.odometerOut ?? wo?.odometerIn ?? 0);
      additionalRecommendations = previous?.additionalRecommendations ?? "";
    }
  });

  function plannedDefaultIso(): string {
    const d = new Date();
    d.setDate(d.getDate() + 3);
    d.setHours(18, 0, 0, 0);
    return d.toISOString();
  }

  function validate(): string | null {
    if (!skipPerformerSignature && performerWorkerId == null) {
      return "Оберіть виконавця";
    }
    if (isNaryad) {
      if (!skipDate && !receivedAt)
        return "Вкажіть дату та час прийняття автомобіля";
      if (!plannedCompletionAt)
        return "Вкажіть орієнтовний термін виконання робіт";
    }
    if (isAkt) {
      if (!skipLocation && !location.trim())
        return "Вкажіть місце складання акту";
      if (!skipDate && !completedAt) return "Вкажіть дату складання акту";
      const raw = odometerOut.trim();
      const parsed = Number(raw);
      if (raw === "" || !Number.isInteger(parsed) || parsed < 0) {
        return "Вкажіть пробіг на момент видачі автомобіля";
      }
    }
    return null;
  }

  function buildOptions(): GenerateDocumentOptions {
    const parseIso = (value: string) =>
      value ? new Date(value).toISOString() : null;
    const rawOdometer = Number(odometerOut.trim());
    return {
      performerWorkerId,
      receivedAt: parseIso(receivedAt),
      plannedCompletionAt: parseIso(plannedCompletionAt),
      location: location.trim() || null,
      completedAt: completedAt ? new Date(completedAt).toISOString() : null,
      odometerOut: isAkt && Number.isInteger(rawOdometer) ? rawOdometer : null,
      additionalRecommendations: additionalRecommendations.trim() || null,
      skipDate,
      skipLocation: isAkt && skipLocation,
      skipPerformerSignature,
      skipCustomerSignature,
    };
  }

  async function submit() {
    const validationError = validate();
    if (validationError) {
      error = validationError;
      return;
    }
    error = null;
    generating = true;
    try {
      const outcome = await generateDocumentMutation.mutateAsync({
        workOrderId,
        kind,
        options: buildOptions(),
      });
      const description = `${isNaryad ? "Наряд-замовлення" : "Акт виконаних робіт"} ${workOrderId}`;
      if (outcome.created) {
        toast.success("Документ створено", {
          description,
          position: "bottom-left",
          action: {
            label: "Відкрити",
            onClick: () =>
              openDocumentMutation.mutate({
                id: outcome.version.id,
                format: "pdf",
              }),
          },
        });
      } else {
        toast.info("Дані не змінились, документ актуальний", {
          description,
          position: "bottom-left",
        });
      }
      open = false;
    } catch {
      // Mutation layer shows the error toast.
    } finally {
      generating = false;
    }
  }
</script>

{#snippet manualToggle(options: {
  pressed: boolean;
  field: string;
  onclick: () => void;
})}
  <AppTooltip content={MANUAL_TOGGLE_TOOLTIP}>
    {#snippet trigger({ props })}
      <Button
        {...props}
        variant={options.pressed ? "default" : "outline"}
        size="icon"
        class="shrink-0"
        aria-pressed={options.pressed}
        aria-label={`${options.field}: ${MANUAL_TOGGLE_TOOLTIP}`}
        onclick={options.onclick}
      >
        <PenLineIcon />
      </Button>
    {/snippet}
  </AppTooltip>
{/snippet}

<Dialog.Root bind:open>
  <Dialog.Content class="gap-4">
    <Dialog.Header>
      <Dialog.Title>
        {isNaryad ? "Наряд-замовлення" : "Акт виконаних робіт"}
      </Dialog.Title>
      <Dialog.Description>
        Вкажіть дані для документа {workOrderId}.
      </Dialog.Description>
    </Dialog.Header>

    <div class="flex min-w-0 flex-col gap-4">
      <div class="flex flex-col gap-1.5">
        <Label>Виконавець</Label>
        <ComboSelect
          items={workersQuery.data ?? []}
          bind:value={performerWorkerId}
          disabled={skipPerformerSignature}
          getKey={(w) => w.id}
          getLabel={workerFullName}
          getSearchValue={(w) => `${workerFullName(w)} ${w.position ?? ""}`}
          placeholder="Оберіть виконавця..."
          searchPlaceholder="Пошук працівників..."
          emptyText="Працівника не знайдено."
        >
          {#snippet trigger(selected)}
            {#if selected}
              <WorkerOption worker={selected} short />
            {:else}
              Оберіть виконавця...
            {/if}
          {/snippet}
          {#snippet item(worker)}
            <WorkerOption {worker} />
          {/snippet}
        </ComboSelect>
      </div>

      {#if isNaryad}
        <div class="flex flex-col gap-1.5">
          <Label for="doc-received-at">Дата та час прийняття</Label>
          <DateTimePicker
            id="doc-received-at"
            bind:value={receivedAt}
            withTime
            placeholder="Оберіть дату та час"
          />
        </div>
        <div class="flex flex-col gap-1.5">
          <Label for="doc-planned-at">Орієнтовний термін виконання</Label>
          <DateTimePicker
            id="doc-planned-at"
            bind:value={plannedCompletionAt}
            withTime
            placeholder="Оберіть дату та час"
          />
        </div>
      {:else}
        <div class="flex flex-col gap-1.5">
          <Label for="doc-location">Місце складання акту</Label>
          <div class="flex items-start gap-2">
            <Textarea
              id="doc-location"
              bind:value={location}
              disabled={skipLocation}
              rows={2}
              class="min-h-8 flex-1"
              placeholder="Наприклад: м. Київ"
            />
            {@render manualToggle({
              pressed: skipLocation,
              field: "Місце складання акту",
              onclick: () => (skipLocation = !skipLocation),
            })}
          </div>
        </div>
        <div class="flex flex-col gap-1.5">
          <Label for="doc-completed-at">Дата складання акту</Label>
          <div class="flex items-center gap-2">
            <DateTimePicker
              id="doc-completed-at"
              bind:value={completedAt}
              disabled={skipDate}
              class="flex-1"
            />
            {@render manualToggle({
              pressed: skipDate,
              field: "Дата складання акту",
              onclick: () => (skipDate = !skipDate),
            })}
          </div>
        </div>
        <div class="flex flex-col gap-1.5">
          <Label for="doc-odometer-out">Пробіг при видачі, км</Label>
          <Input
            id="doc-odometer-out"
            type="text"
            inputmode="numeric"
            bind:value={odometerOut}
          />
        </div>
        <div class="flex flex-col gap-1.5">
          <Label for="doc-additional-recommendations"
            >Додаткові рекомендації</Label
          >
          <Textarea
            id="doc-additional-recommendations"
            bind:value={additionalRecommendations}
            rows={3}
            placeholder="Залиште порожнім, якщо таких немає..."
          />
        </div>
      {/if}

      <div class="flex flex-col gap-1.5">
        <Label>Ініціали підписантів</Label>
        <div
          class="flex flex-col divide-y divide-border rounded-md border border-border"
        >
          <div class="flex items-center gap-3 px-3 py-2.5">
            <span class="min-w-0 flex-1">
              Виконавець{performerSignatureName
                ? ` — ${performerSignatureName}`
                : ""}
            </span>
            {@render manualToggle({
              pressed: skipPerformerSignature,
              field: "Виконавець",
              onclick: () => (skipPerformerSignature = !skipPerformerSignature),
            })}
          </div>
          <div class="flex items-center gap-3 px-3 py-2.5">
            <span class="min-w-0 flex-1">
              Замовник{customerSignatureName
                ? ` — ${customerSignatureName}`
                : ""}
            </span>
            {@render manualToggle({
              pressed: skipCustomerSignature,
              field: "Замовник",
              onclick: () => (skipCustomerSignature = !skipCustomerSignature),
            })}
          </div>
        </div>
      </div>

      {#if error}
        <p class="text-sm text-destructive">{error}</p>
      {/if}
    </div>

    <Dialog.Footer>
      <Button
        variant="outline"
        onclick={() => (open = false)}
        disabled={generating}
      >
        Відмінити
      </Button>
      <Button onclick={submit} disabled={generating}>
        {#if generating}
          <Spinner class="size-4" />
        {/if}
        Створити
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

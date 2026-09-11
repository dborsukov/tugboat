<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import { toast } from "svelte-sonner";

  import { Button } from "$lib/components/ui/button";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import Spinner from "$lib/components/ui/spinner/spinner.svelte";
  import { Textarea } from "$lib/components/ui/textarea";

  import {
    commands,
    type DocumentConflictAction,
    type DocumentFormat,
    type DocumentKind,
    type GenerateDocumentOptions,
    type WorkOrderDetailsDto,
  } from "$lib/bindings";
  import { settingsState } from "$lib/states/settings.svelte";
  import { unwrap } from "$lib/tools";

  import { createGenerateDocumentMutation } from "../queries";

  let {
    workOrderId,
    workOrderDetails,
    kind,
    format,
    open = $bindable(false),
  }: {
    workOrderId: string;
    workOrderDetails: WorkOrderDetailsDto | undefined;
    kind: DocumentKind;
    format: DocumentFormat;
    open: boolean;
  } = $props();

  const queryClient = useQueryClient();
  const generateDocumentMutation = createGenerateDocumentMutation(queryClient);

  const isNaryad = $derived(kind === "naryad_zamovlennya");
  const isAkt = $derived(kind === "akt_vykonanykh_robit");

  // ---------- Form state ----------

  let performerSurname = $state("");
  let performerFirstName = $state("");
  let performerPatronymic = $state("");
  let performerPosition = $state("");
  let skipPerformerSignature = $state(false);
  let skipCustomerSignature = $state(false);
  let receivedAt = $state("");
  let plannedCompletionAt = $state("");
  let skipDate = $state(false);
  let location = $state("");
  let completedAt = $state("");
  let skipLocation = $state(false);
  let odometerOut = $state("");
  let refusedDefects = $state("");

  let error = $state<string | null>(null);

  let isConflictOpen = $state(false);
  let generating = $state(false);

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

  $effect(() => {
    if (open) {
      error = null;
      isConflictOpen = false;
      generating = false;

      const wo = workOrderDetails?.workOrder;
      performerSurname = "";
      performerFirstName = "";
      performerPatronymic = "";
      performerPosition = "";
      skipPerformerSignature = false;
      skipCustomerSignature = false;
      receivedAt = toLocalDateTimeValue(wo?.createdAt);
      plannedCompletionAt = toLocalDateTimeValue(plannedDefaultIso());
      skipDate = false;
      location = settingsState.settings?.companyInfo?.physicalAddress ?? "";
      completedAt = toLocalDateValue(new Date().toISOString());
      skipLocation = false;
      odometerOut = String(wo?.odometerOut ?? wo?.odometerIn ?? 0);
      refusedDefects = "";
    }
  });

  function plannedDefaultIso(): string {
    const d = new Date();
    d.setDate(d.getDate() + 3);
    d.setHours(18, 0, 0, 0);
    return d.toISOString();
  }

  function validate(): string | null {
    if (
      !skipPerformerSignature &&
      (!performerSurname.trim() || !performerFirstName.trim())
    ) {
      return "Вкажіть ПІБ виконавця";
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
      performerSurname: performerSurname.trim() || null,
      performerFirstName: performerFirstName.trim() || null,
      performerPatronymic: performerPatronymic.trim() || null,
      performerPosition: performerPosition.trim() || null,
      receivedAt: parseIso(receivedAt),
      plannedCompletionAt: parseIso(plannedCompletionAt),
      location: location.trim() || null,
      completedAt: completedAt ? new Date(completedAt).toISOString() : null,
      odometerOut: isAkt && Number.isInteger(rawOdometer) ? rawOdometer : null,
      refusedDefects: refusedDefects.trim() || null,
      skipDate,
      skipLocation: isAkt && skipLocation,
      skipPerformerSignature,
      skipCustomerSignature,
    };
  }

  async function submit(conflict: DocumentConflictAction) {
    const validationError = validate();
    if (validationError) {
      error = validationError;
      return;
    }
    error = null;
    isConflictOpen = false;
    generating = true;
    try {
      const document = await generateDocumentMutation.mutateAsync({
        workOrderId,
        kind,
        format,
        options: buildOptions(),
        conflict,
      });
      toast.success("Документ збережено", {
        description: document.name,
        position: "bottom-left",
      });
      open = false;
    } catch {
      // Mutation layer shows the error toast.
    } finally {
      generating = false;
    }
  }

  async function handleGenerate() {
    const validationError = validate();
    if (validationError) {
      error = validationError;
      return;
    }
    error = null;

    try {
      const existing = await unwrap(
        commands.documentTargetPath(workOrderId, kind, format),
      );
      if (existing) {
        isConflictOpen = true;
        return;
      }
      await submit("replace");
    } catch {
      // Error toast is shown by the mutation layer on submit failure.
    }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="gap-4">
    <Dialog.Header>
      <Dialog.Title>
        {isNaryad ? "Наряд-замовлення" : "Акт виконаних робіт"} —{" "}
        {format.toUpperCase()}
      </Dialog.Title>
      <Dialog.Description>
        Вкажіть додаткові дані для документа {workOrderId}.
      </Dialog.Description>
    </Dialog.Header>

    <div class="flex flex-col gap-3">
      <div class="flex flex-col gap-1.5">
        <Label>Виконавець *</Label>
        <div class="grid grid-cols-1 gap-2 sm:grid-cols-3">
          <div class="flex flex-col gap-1.5">
            <Label
              for="doc-performer-surname"
              class="text-xs text-muted-foreground"
            >
              Прізвище
            </Label>
            <Input
              id="doc-performer-surname"
              placeholder="Прізвище"
              bind:value={performerSurname}
              disabled={skipPerformerSignature}
            />
          </div>
          <div class="flex flex-col gap-1.5">
            <Label
              for="doc-performer-first-name"
              class="text-xs text-muted-foreground"
            >
              Ім'я
            </Label>
            <Input
              id="doc-performer-first-name"
              placeholder="Ім'я"
              bind:value={performerFirstName}
              disabled={skipPerformerSignature}
            />
          </div>
          <div class="flex flex-col gap-1.5">
            <Label
              for="doc-performer-patronymic"
              class="text-xs text-muted-foreground"
            >
              По-батькові
            </Label>
            <Input
              id="doc-performer-patronymic"
              placeholder="По-батькові"
              bind:value={performerPatronymic}
              disabled={skipPerformerSignature}
            />
          </div>
        </div>
      </div>
      <div class="flex flex-col gap-1.5">
        <Label for="doc-performer-position">Посада</Label>
        <Input
          id="doc-performer-position"
          placeholder="Наприклад: приймальник"
          bind:value={performerPosition}
          disabled={skipPerformerSignature}
        />
      </div>
      <div class="flex items-center gap-2">
        <Checkbox
          id="doc-skip-performer-signature"
          checked={skipPerformerSignature}
          onCheckedChange={(checked) =>
            (skipPerformerSignature = checked === true)}
        />
        <Label for="doc-skip-performer-signature" class="font-normal">
          Без підпису виконавця (заповнити від руки після друку)
        </Label>
      </div>
      <div class="flex items-center gap-2">
        <Checkbox
          id="doc-skip-customer-signature"
          checked={skipCustomerSignature}
          onCheckedChange={(checked) =>
            (skipCustomerSignature = checked === true)}
        />
        <Label for="doc-skip-customer-signature" class="font-normal">
          Без підпису замовника (заповнити від руки після друку)
        </Label>
      </div>

      {#if isNaryad}
        <div class="flex flex-col gap-1.5">
          <Label for="doc-received-at">Дата та час прийняття *</Label>
          <Input
            id="doc-received-at"
            type="datetime-local"
            bind:value={receivedAt}
          />
        </div>
        <div class="flex flex-col gap-1.5">
          <Label for="doc-planned-at">Орієнтовний термін виконання *</Label>
          <Input
            id="doc-planned-at"
            type="datetime-local"
            bind:value={plannedCompletionAt}
          />
        </div>
      {:else}
        <div class="flex flex-col gap-1.5">
          <Label for="doc-location">Місце складання акту *</Label>
          <Input
            id="doc-location"
            bind:value={location}
            disabled={skipLocation}
            placeholder="Наприклад: м. Київ"
          />
        </div>
        <div class="flex items-center gap-2">
          <Checkbox
            id="doc-skip-location"
            checked={skipLocation}
            onCheckedChange={(checked) => (skipLocation = checked === true)}
          />
          <Label for="doc-skip-location" class="font-normal">
            Без місця складання (заповнити від руки після друку)
          </Label>
        </div>
        <div class="flex flex-col gap-1.5">
          <Label for="doc-completed-at">Дата складання акту *</Label>
          <Input
            id="doc-completed-at"
            type="date"
            bind:value={completedAt}
            disabled={skipDate}
          />
        </div>
        <div class="flex items-center gap-2">
          <Checkbox
            id="doc-skip-date"
            checked={skipDate}
            onCheckedChange={(checked) => (skipDate = checked === true)}
          />
          <Label for="doc-skip-date" class="font-normal">
            Без дати (заповнити від руки після друку)
          </Label>
        </div>
        <div class="flex flex-col gap-1.5">
          <Label for="doc-odometer-out">Пробіг при видачі, км *</Label>
          <Input
            id="doc-odometer-out"
            type="text"
            inputmode="numeric"
            bind:value={odometerOut}
          />
        </div>
        <div class="flex flex-col gap-1.5">
          <Label for="doc-refused">
            Дефекти або рекомендації, від усунення яких замовник відмовився
          </Label>
          <Textarea
            id="doc-refused"
            bind:value={refusedDefects}
            rows={3}
            placeholder="Залиште порожнім, якщо таких немає..."
          />
        </div>
      {/if}

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
      <Button onclick={handleGenerate} disabled={generating}>
        {#if generating}
          <Spinner class="size-4" />
        {/if}
        Згенерувати
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<Dialog.Root bind:open={isConflictOpen}>
  <Dialog.Content class="sm:max-w-sm">
    <Dialog.Header>
      <Dialog.Title>Файл уже існує</Dialog.Title>
      <Dialog.Description>
        Документ із таким іменем уже збережено. Замінити наявний файл або
        зберегти поруч під новим іменем?
      </Dialog.Description>
    </Dialog.Header>
    <Dialog.Footer>
      <Button variant="outline" onclick={() => (isConflictOpen = false)}>
        Скасувати
      </Button>
      <Button variant="outline" onclick={() => submit("rename")}>
        Перейменувати
      </Button>
      <Button onclick={() => submit("replace")}>Замінити</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

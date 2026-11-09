<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import { tick } from "svelte";
  import type { CountryCode } from "svelte-tel-input/types";
  import { validateTelInput } from "svelte-tel-input/validators";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import CopyIcon from "@lucide/svelte/icons/copy";

  import { Button } from "$lib/components/ui/button";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Textarea } from "$lib/components/ui/textarea";

  import PhoneInput from "$lib/components/phone-input.svelte";
  import {
    sheetInput,
    sheetLabel,
    sheetSecondaryButton,
    sheetTextarea,
  } from "$lib/components/sheet/field-styles";
  import NotesDisclosure from "$lib/components/sheet/notes-disclosure.svelte";
  import {
    interpretQuery,
    toInternationalPhone,
  } from "$lib/components/sheet/query-prefill";
  import RequiredMark from "$lib/components/sheet/required-mark.svelte";
  import SegmentedControl from "$lib/components/sheet/segmented-control.svelte";
  import SheetFrame from "$lib/components/sheet/sheet-frame.svelte";
  import SheetSection from "$lib/components/sheet/sheet-section.svelte";

  import type {
    ClientListItem,
    ClientPayload,
    ClientType,
  } from "$lib/bindings";
  import { settingsState } from "$lib/states/settings.svelte";
  import { cn } from "$lib/utils";

  import {
    createClientDetailsQuery,
    createClientsQuery,
    createCreateClientMutation,
    createUpdateClientMutation,
  } from "./queries";

  let {
    active = true,
    clientId = null,
    initialQuery = "",
    onCancel,
    onCreated,
    onBack,
    backLabel,
    stepLabel,
  }: {
    active?: boolean;
    clientId?: number | null;
    initialQuery?: string;
    onCancel: () => void;
    onCreated: (id: number, name?: string) => void;
    onBack?: () => void;
    backLabel?: string;
    stepLabel?: string;
  } = $props();

  const formID = "client-sheet-form";
  const isEdit = $derived(clientId != null);

  const queryClient = useQueryClient();
  const createClientMutation = createCreateClientMutation(queryClient);
  const updateClientMutation = createUpdateClientMutation(queryClient);

  const clientsQuery = createClientsQuery();
  const clientsList = $derived(clientsQuery.data ?? []);
  const client = $derived(clientsList.find((c) => c.id === clientId) ?? null);

  const clientDetailsQuery = createClientDetailsQuery(() =>
    isEdit ? clientId : null,
  );
  const clientDetails = $derived(clientDetailsQuery.data?.details ?? null);

  let phoneCountry = $state<CountryCode | null>(null);
  let appliedDefaultCountry: CountryCode | null = null;

  $effect(() => {
    const configured = (settingsState.settings?.defaultPhoneCountryCode ??
      "UA") as CountryCode;
    if (phoneCountry === appliedDefaultCountry) {
      phoneCountry = configured;
      appliedDefaultCountry = configured;
    }
  });

  const TYPE_LABELS: Record<ClientType, string> = {
    individual: "Фізична особа",
    fop: "ФОП",
    company: "Організація",
  };

  const TYPE_OPTIONS = (["individual", "company", "fop"] as ClientType[]).map(
    (value) => ({ value, label: TYPE_LABELS[value] }),
  );

  const clientSchema = z
    .object({
      clientType: z.enum(["individual", "fop", "company"]),
      name: z.string().default(""),
      surname: z.string().default(""),
      firstName: z.string().default(""),
      patronymic: z.string().default(""),
      phone: z.string().default(""),
      email: z.string().default(""),
      legalName: z.string().default(""),
      legalAddress: z.string().default(""),
      edrpou: z.string().default(""),
      rnokpp: z.string().default(""),
      directorSurname: z.string().default(""),
      directorFirstName: z.string().default(""),
      directorPatronymic: z.string().default(""),
      contactSurname: z.string().default(""),
      contactFirstName: z.string().default(""),
      contactPatronymic: z.string().default(""),
      contactPhone: z.string().default(""),
      contactEmail: z.string().default(""),
      iban: z.string().default(""),
      bankName: z.string().default(""),
      vatPayer: z.boolean().default(false),
      notes: z.string().default(""),
    })
    .superRefine((data, ctx) => {
      const phone = data.phone.trim();

      if (data.clientType === "company") {
        if (data.name.trim() === "") {
          ctx.addIssue({
            code: "custom",
            path: ["name"],
            message: "Обов'язкове поле",
          });
        }
      } else if (data.firstName.trim() === "") {
        ctx.addIssue({
          code: "custom",
          path: ["firstName"],
          message: "Обов'язкове поле",
        });
      }

      if (
        data.clientType === "company" &&
        requireFirstName(
          data.directorSurname,
          data.directorFirstName,
          data.directorPatronymic,
        )
      ) {
        ctx.addIssue({
          code: "custom",
          path: ["directorFirstName"],
          message: "Вкажіть ім'я керівника",
        });
      }
      if (
        data.clientType !== "individual" &&
        requireFirstName(
          data.contactSurname,
          data.contactFirstName,
          data.contactPatronymic,
        )
      ) {
        ctx.addIssue({
          code: "custom",
          path: ["contactFirstName"],
          message: "Вкажіть ім'я контактної особи",
        });
      }

      if (data.clientType === "individual") {
        const error = validateTelInput(data.phone, {
          required: true,
          country: phoneCountry,
        });
        if (error) {
          ctx.addIssue({
            code: "custom",
            path: ["phone"],
            message:
              error === "REQUIRED" ? "Обов'язкове поле" : "Неправильний формат",
          });
        }
      }

      if (phone !== "") {
        const duplicateClient = clientsList.find((c) => c.phone === phone);
        if (duplicateClient && !(isEdit && duplicateClient.id === client?.id)) {
          ctx.addIssue({
            code: "custom",
            path: ["phone"],
            message: "Клієнт з таким номером вже існує",
          });
        }
      }

      const email = data.email.trim();
      if (email !== "" && !/^\S+@\S+\.\S+$/.test(email)) {
        ctx.addIssue({
          code: "custom",
          path: ["email"],
          message: "Неправильний email",
        });
      }

      const contactEmail = data.contactEmail.trim();
      if (contactEmail !== "" && !/^\S+@\S+\.\S+$/.test(contactEmail)) {
        ctx.addIssue({
          code: "custom",
          path: ["contactEmail"],
          message: "Неправильний email",
        });
      }

      const selfId = isEdit ? (client?.id ?? null) : null;

      if (hasDuplicateValue(clientsList, (c) => c.email, email, selfId)) {
        ctx.addIssue({
          code: "custom",
          path: ["email"],
          message: "Клієнт з таким email вже існує",
        });
      }

      if (
        hasDuplicateValue(
          clientsList,
          (c) => c.contactEmail,
          contactEmail,
          selfId,
        )
      ) {
        ctx.addIssue({
          code: "custom",
          path: ["contactEmail"],
          message: "Клієнт з таким email вже існує",
        });
      }

      if (data.clientType === "fop") {
        if (
          hasDuplicateValue(
            clientsList,
            (c) => c.rnokpp,
            data.rnokpp.trim(),
            selfId,
          )
        ) {
          ctx.addIssue({
            code: "custom",
            path: ["rnokpp"],
            message: "Клієнт з таким РНОКПП вже існує",
          });
        }
      }

      if (data.clientType === "company") {
        if (
          hasDuplicateValue(
            clientsList,
            (c) => c.edrpou,
            data.edrpou.trim(),
            selfId,
          )
        ) {
          ctx.addIssue({
            code: "custom",
            path: ["edrpou"],
            message: "Клієнт з таким ЄДРПОУ вже існує",
          });
        }
      }

      if (data.clientType !== "individual") {
        if (
          hasDuplicateValue(
            clientsList,
            (c) => c.iban,
            data.iban.trim(),
            selfId,
          )
        ) {
          ctx.addIssue({
            code: "custom",
            path: ["iban"],
            message: "Клієнт з таким IBAN вже існує",
          });
        }
      }

      if (data.clientType === "fop") {
        if (!/^\d{10}$/.test(data.rnokpp.trim())) {
          ctx.addIssue({
            code: "custom",
            path: ["rnokpp"],
            message: "РНОКПП — 10 цифр",
          });
        }
      } else if (data.clientType === "company") {
        if (data.legalName.trim() === "") {
          ctx.addIssue({
            code: "custom",
            path: ["legalName"],
            message: "Обов'язкове поле",
          });
        }
        if (data.legalAddress.trim() === "") {
          ctx.addIssue({
            code: "custom",
            path: ["legalAddress"],
            message: "Обов'язкове поле",
          });
        }
        if (!/^\d{8}$/.test(data.edrpou.trim())) {
          ctx.addIssue({
            code: "custom",
            path: ["edrpou"],
            message: "ЄДРПОУ — 8 цифр",
          });
        }
      }

      const ibanError = getIbanError(data.iban);
      if (ibanError) {
        ctx.addIssue({ code: "custom", path: ["iban"], message: ibanError });
      }
    });

  type ClientFormData = z.infer<typeof clientSchema>;

  function requireFirstName(
    surname: string,
    firstName: string,
    patronymic: string,
  ): boolean {
    const hasSomeData = surname.trim() !== "" || patronymic.trim() !== "";
    return hasSomeData && firstName.trim() === "";
  }

  function getIbanError(iban: string): string | null {
    const value = iban.trim();
    if (value !== "" && !/^[A-Z]{2}\d{2}[A-Z0-9]{11,30}$/.test(value)) {
      return "Неправильний IBAN";
    }
    return null;
  }

  function trimToNull(value: string): string | null {
    const trimmed = value.trim();
    return trimmed === "" ? null : trimmed;
  }

  function hasDuplicateValue(
    list: ClientListItem[],
    getValue: (c: ClientListItem) => string | null,
    value: string,
    selfId: number | null,
  ): boolean {
    if (value === "") return false;
    const normalized = value.toLowerCase();
    return list.some(
      (c) =>
        c.id !== selfId && (getValue(c) ?? "").toLowerCase() === normalized,
    );
  }

  function buildPayload(data: ClientFormData): ClientPayload {
    const contactCommon = {
      notes: trimToNull(data.notes),
      contactSurname: trimToNull(data.contactSurname),
      contactFirstName: trimToNull(data.contactFirstName),
      contactPatronymic: trimToNull(data.contactPatronymic),
      contactPhone: trimToNull(data.contactPhone),
      contactEmail: trimToNull(data.contactEmail),
      iban: trimToNull(data.iban),
      bankName: trimToNull(data.bankName),
    };

    switch (data.clientType) {
      case "fop":
        return {
          clientType: "fop",
          surname: trimToNull(data.surname),
          firstName: data.firstName.trim(),
          patronymic: trimToNull(data.patronymic),
          ...contactCommon,
          rnokpp: data.rnokpp.trim(),
          legalAddress: trimToNull(data.legalAddress),
          vatPayer: data.vatPayer,
        };
      case "company":
        return {
          clientType: "company",
          name: data.name.trim(),
          ...contactCommon,
          legalName: data.legalName.trim(),
          legalAddress: data.legalAddress.trim(),
          edrpou: data.edrpou.trim(),
          directorSurname: trimToNull(data.directorSurname),
          directorFirstName: trimToNull(data.directorFirstName),
          directorPatronymic: trimToNull(data.directorPatronymic),
          vatPayer: data.vatPayer,
        };
      default:
        return {
          clientType: "individual",
          surname: trimToNull(data.surname),
          firstName: data.firstName.trim(),
          patronymic: trimToNull(data.patronymic),
          phone: trimToNull(data.phone),
          email: trimToNull(data.email),
          notes: trimToNull(data.notes),
        };
    }
  }

  function copyFopAsContact() {
    $formData.contactSurname = $formData.surname;
    $formData.contactFirstName = $formData.firstName;
    $formData.contactPatronymic = $formData.patronymic;
  }

  function copyDirectorAsContact() {
    $formData.contactSurname = $formData.directorSurname;
    $formData.contactFirstName = $formData.directorFirstName;
    $formData.contactPatronymic = $formData.directorPatronymic;
  }

  let firstFieldEl = $state<HTMLInputElement | null>(null);
  let notesOpen = $state(false);
  let baseline = $state<string | null>(null);
  let loadedKey = $state<string | null>(null);
  let submitError = $state<string | null>(null);

  function captureSubmitError(error: unknown) {
    submitError =
      error instanceof Error ? error.message : "Не вдалося зберегти";
    return null;
  }

  function markBaseline() {
    baseline = JSON.stringify($formData);
  }

  function focusFirstField() {
    firstFieldEl?.focus();
    setTimeout(() => firstFieldEl?.focus(), 120);
  }

  const form = superForm(defaults(zod4(clientSchema)), {
    SPA: true,
    validators: zod4Client(clientSchema),
    validationMethod: "oninput",
    resetForm: false,
    id: formID,
    async onUpdate({ form: nativeForm, cancel }) {
      if (!nativeForm.valid) return;

      const payload = buildPayload(nativeForm.data);
      submitError = null;
      const result =
        isEdit && clientId != null
          ? await updateClientMutation
              .mutateAsync({ id: clientId, payload })
              .catch(captureSubmitError)
          : await createClientMutation
              .mutateAsync(payload)
              .catch(captureSubmitError);

      if (!result) return;
      cancel();
      baseline = null;
      form.reset();
      loadedKey = null;
      onCreated(result.id, result.name);
    },
  });

  const { form: formData, enhance, submitting } = form;

  const dirty = $derived(
    baseline !== null && JSON.stringify($formData) !== baseline,
  );

  function applyQueryPrefill(query: string) {
    const interpreted = interpretQuery(query);
    if (interpreted.kind === "phone") {
      $formData.phone = toInternationalPhone(interpreted.value, phoneCountry);
    } else if (interpreted.kind === "text") {
      $formData.surname = interpreted.words[0] ?? "";
    }
  }

  $effect(() => {
    if (!active) {
      loadedKey = null;
      submitError = null;
      return;
    }

    const key = `${isEdit ? clientId : "new"}:${initialQuery}`;
    if (loadedKey === key) return;
    if (isEdit && (clientsQuery.isPending || clientDetailsQuery.isPending)) {
      return;
    }

    loadedKey = key;

    if (client) {
      $formData.clientType = client.clientType;
      $formData.name = client.name ?? "";
      $formData.surname = client.surname ?? "";
      $formData.firstName = client.firstName ?? "";
      $formData.patronymic = client.patronymic ?? "";
      $formData.phone = client.phone ?? "";
      $formData.email = client.email ?? "";
      $formData.notes = client.notes ?? "";

      const details = clientDetails;
      $formData.legalName = details?.legalName ?? "";
      $formData.legalAddress = details?.legalAddress ?? "";
      $formData.edrpou = details?.edrpou ?? "";
      $formData.rnokpp = details?.rnokpp ?? "";
      $formData.directorSurname = details?.directorSurname ?? "";
      $formData.directorFirstName = details?.directorFirstName ?? "";
      $formData.directorPatronymic = details?.directorPatronymic ?? "";
      $formData.contactSurname = details?.contactSurname ?? "";
      $formData.contactFirstName = details?.contactFirstName ?? "";
      $formData.contactPatronymic = details?.contactPatronymic ?? "";
      $formData.contactPhone = details?.contactPhone ?? "";
      $formData.contactEmail = details?.contactEmail ?? "";
      $formData.iban = details?.iban ?? "";
      $formData.bankName = details?.bankName ?? "";
      $formData.vatPayer = details?.vatPayer ?? false;
      notesOpen = (client.notes ?? "").trim() !== "";
    } else {
      form.reset();
      notesOpen = false;
      applyQueryPrefill(initialQuery);
      $formData.clientType = "individual";
    }

    void tick().then(() => {
      markBaseline();
      focusFirstField();
    });
  });

  function handleCancel() {
    form.reset();
    baseline = null;
    loadedKey = null;
    submitError = null;
    onCancel();
  }
</script>

{#snippet requiredMark()}
  <RequiredMark />
{/snippet}

{#snippet pibRow(
  surnameField: "surname" | "directorSurname" | "contactSurname",
  firstNameField: "firstName" | "directorFirstName" | "contactFirstName",
  patronymicField: "patronymic" | "directorPatronymic" | "contactPatronymic",
  requiredFirstName = false,
)}
  <div class="grid grid-cols-1 gap-x-3 gap-y-4 sm:grid-cols-3">
    <Form.Field {form} name={surnameField} class="space-y-1.5">
      <Form.Control>
        {#snippet children({ props })}
          <Label class={sheetLabel}>Прізвище</Label>
          <Input
            {...props}
            class={sheetInput}
            bind:value={$formData[surnameField]}
          />
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>

    <Form.Field {form} name={firstNameField} class="space-y-1.5">
      <Form.Control>
        {#snippet children({ props })}
          <Label class={sheetLabel}>
            Ім'я{#if requiredFirstName}{@render requiredMark()}{/if}
          </Label>
          <Input
            {...props}
            class={sheetInput}
            bind:value={$formData[firstNameField]}
          />
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>

    <Form.Field {form} name={patronymicField} class="space-y-1.5">
      <Form.Control>
        {#snippet children({ props })}
          <Label class={sheetLabel}>По батькові</Label>
          <Input
            {...props}
            class={sheetInput}
            bind:value={$formData[patronymicField]}
          />
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>
  </div>
{/snippet}

<SheetFrame
  {formID}
  {active}
  {dirty}
  title={isEdit ? "Редагувати клієнта" : "Створити клієнта"}
  subtitle={isEdit ? "Змініть дані клієнта" : "Внесіть дані нового клієнта"}
  submitLabel={isEdit ? "Зберегти" : "Створити"}
  shortcutLabel={isEdit ? "зберегти" : "створити"}
  submitting={$submitting}
  error={submitError}
  enhanceAction={enhance}
  onRequestClose={handleCancel}
  {onBack}
  {backLabel}
  {stepLabel}
  class="gap-0"
>
  <SheetSection title="Тип клієнта" class="pt-0">
    <SegmentedControl
      options={TYPE_OPTIONS}
      value={$formData.clientType}
      clearable={false}
      ariaLabel="Тип клієнта"
      onValueChange={(value) => {
        if (value) $formData.clientType = value as ClientType;
      }}
    />
  </SheetSection>

  <SheetSection title="Основне">
    {#if $formData.clientType === "company"}
      <Form.Field {form} name="name" class="space-y-1.5">
        <Form.Control>
          {#snippet children({ props })}
            <Label class={sheetLabel}>
              Коротка назва{@render requiredMark()}
            </Label>
            <Input
              {...props}
              bind:ref={firstFieldEl}
              data-sheet-autofocus
              class={sheetInput}
              bind:value={$formData.name}
              placeholder="напр. Автосервіс Плюс"
            />
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>
    {:else}
      <div class="grid grid-cols-1 gap-x-3 gap-y-4 sm:grid-cols-3">
        <Form.Field {form} name="surname" class="space-y-1.5">
          <Form.Control>
            {#snippet children({ props })}
              <Label class={sheetLabel}>Прізвище</Label>
              <Input
                {...props}
                bind:ref={firstFieldEl}
                data-sheet-autofocus
                class={sheetInput}
                bind:value={$formData.surname}
              />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>

        <Form.Field {form} name="firstName" class="space-y-1.5">
          <Form.Control>
            {#snippet children({ props })}
              <Label class={sheetLabel}>
                Ім'я{@render requiredMark()}
              </Label>
              <Input
                {...props}
                class={sheetInput}
                bind:value={$formData.firstName}
              />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>

        <Form.Field {form} name="patronymic" class="space-y-1.5">
          <Form.Control>
            {#snippet children({ props })}
              <Label class={sheetLabel}>По батькові</Label>
              <Input
                {...props}
                class={sheetInput}
                bind:value={$formData.patronymic}
              />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>
      </div>
    {/if}
  </SheetSection>

  {#if $formData.clientType === "individual"}
    <SheetSection title="Контакти">
      <div class="flex flex-col gap-4">
        <Form.Field {form} name="phone" class="space-y-1.5">
          <Form.Control>
            {#snippet children({ props })}
              <Label class={sheetLabel}>
                Мобільний номер{@render requiredMark()}
              </Label>
              <PhoneInput
                {...props}
                class={sheetInput}
                triggerClass="h-9.5 rounded-[8px] border border-border bg-muted/40 px-2.5 focus-visible:border-brand focus-visible:ring-3 focus-visible:ring-brand/20"
                bind:value={$formData.phone}
                bind:country={phoneCountry}
              />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>

        <Form.Field {form} name="email" class="space-y-1.5">
          <Form.Control>
            {#snippet children({ props })}
              <Label class={sheetLabel}>Email</Label>
              <Input
                {...props}
                class={sheetInput}
                bind:value={$formData.email}
                type="email"
                placeholder="email@example.com"
              />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>
      </div>
    </SheetSection>
  {:else if $formData.clientType === "company"}
    <SheetSection title="Дані організації">
      <div class="flex flex-col gap-4">
        <Form.Field {form} name="legalName" class="space-y-1.5">
          <Form.Control>
            {#snippet children({ props })}
              <Label class={sheetLabel}>
                Юридична назва{@render requiredMark()}
              </Label>
              <Input
                {...props}
                class={sheetInput}
                bind:value={$formData.legalName}
                placeholder="напр. ТОВ «Автосервіс Плюс»"
              />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>

        <div class="grid grid-cols-1 gap-x-3 gap-y-4 sm:grid-cols-2">
          <Form.Field {form} name="edrpou" class="space-y-1.5">
            <Form.Control>
              {#snippet children({ props })}
                <Label class={sheetLabel}>
                  ЄДРПОУ{@render requiredMark()}
                </Label>
                <Input
                  {...props}
                  class={sheetInput}
                  bind:value={$formData.edrpou}
                  inputmode="numeric"
                  placeholder="12345678"
                />
              {/snippet}
            </Form.Control>
            <Form.FieldErrors />
          </Form.Field>

          <div class="flex items-end pb-2">
            <Form.Field {form} name="vatPayer" class="space-y-1.5">
              <Form.Control>
                {#snippet children({ props })}
                  <div class="flex items-center gap-2">
                    <Checkbox
                      {...props}
                      id="vatPayer"
                      bind:checked={$formData.vatPayer}
                    />
                    <Label for="vatPayer" class="text-[12.5px] font-normal">
                      Платник ПДВ
                    </Label>
                  </div>
                {/snippet}
              </Form.Control>
              <Form.FieldErrors />
            </Form.Field>
          </div>
        </div>

        <Form.Field {form} name="legalAddress" class="space-y-1.5">
          <Form.Control>
            {#snippet children({ props })}
              <Label class={sheetLabel}>
                Юридична адреса{@render requiredMark()}
              </Label>
              <Textarea
                {...props}
                class={sheetTextarea}
                bind:value={$formData.legalAddress}
                placeholder="Адреса реєстрації організації..."
                rows={2}
              />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>
      </div>
    </SheetSection>

    <SheetSection title="Керівник">
      {@render pibRow(
        "directorSurname",
        "directorFirstName",
        "directorPatronymic",
      )}
    </SheetSection>
  {:else}
    <SheetSection title="Дані ФОП">
      <div class="flex flex-col gap-4">
        <div class="grid grid-cols-1 gap-x-3 gap-y-4 sm:grid-cols-2">
          <Form.Field {form} name="rnokpp" class="space-y-1.5">
            <Form.Control>
              {#snippet children({ props })}
                <Label class={sheetLabel}>
                  РНОКПП{@render requiredMark()}
                </Label>
                <Input
                  {...props}
                  class={sheetInput}
                  bind:value={$formData.rnokpp}
                  inputmode="numeric"
                  placeholder="1234567890"
                />
              {/snippet}
            </Form.Control>
            <Form.FieldErrors />
          </Form.Field>

          <div class="flex items-end pb-2">
            <Form.Field {form} name="vatPayer" class="space-y-1.5">
              <Form.Control>
                {#snippet children({ props })}
                  <div class="flex items-center gap-2">
                    <Checkbox
                      {...props}
                      id="vatPayerFop"
                      bind:checked={$formData.vatPayer}
                    />
                    <Label for="vatPayerFop" class="text-[12.5px] font-normal">
                      Платник ПДВ
                    </Label>
                  </div>
                {/snippet}
              </Form.Control>
              <Form.FieldErrors />
            </Form.Field>
          </div>
        </div>

        <Form.Field {form} name="legalAddress" class="space-y-1.5">
          <Form.Control>
            {#snippet children({ props })}
              <Label class={sheetLabel}>Адреса реєстрації</Label>
              <Textarea
                {...props}
                class={sheetTextarea}
                bind:value={$formData.legalAddress}
                placeholder="Адреса реєстрації ФОП..."
                rows={2}
              />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>
      </div>
    </SheetSection>
  {/if}

  {#if $formData.clientType !== "individual"}
    <SheetSection title="Контактна особа">
      <div class="flex flex-col gap-4">
        <Button
          variant="outline"
          class={cn(sheetSecondaryButton, "w-full justify-start")}
          type="button"
          onclick={$formData.clientType === "company"
            ? copyDirectorAsContact
            : copyFopAsContact}
        >
          <CopyIcon class="size-3.5" />
          {$formData.clientType === "company"
            ? "Копіювати від керівника"
            : "Копіювати з ФОП"}
        </Button>

        {@render pibRow(
          "contactSurname",
          "contactFirstName",
          "contactPatronymic",
        )}

        <Form.Field {form} name="contactPhone" class="space-y-1.5">
          <Form.Control>
            {#snippet children({ props })}
              <Label class={sheetLabel}>Телефон контактної особи</Label>
              <PhoneInput
                {...props}
                class={sheetInput}
                triggerClass="h-9.5 rounded-[8px] border border-border bg-muted/40 px-2.5 focus-visible:border-brand focus-visible:ring-3 focus-visible:ring-brand/20"
                bind:value={$formData.contactPhone}
                bind:country={phoneCountry}
              />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>

        <Form.Field {form} name="contactEmail" class="space-y-1.5">
          <Form.Control>
            {#snippet children({ props })}
              <Label class={sheetLabel}>Email контактної особи</Label>
              <Input
                {...props}
                class={sheetInput}
                bind:value={$formData.contactEmail}
                type="email"
                placeholder="email@example.com"
              />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>
      </div>
    </SheetSection>

    <SheetSection title="Реквізити">
      <div class="flex flex-col gap-4">
        <Form.Field {form} name="bankName" class="space-y-1.5">
          <Form.Control>
            {#snippet children({ props })}
              <Label class={sheetLabel}>Назва банку</Label>
              <Input
                {...props}
                class={sheetInput}
                bind:value={$formData.bankName}
              />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>

        <Form.Field {form} name="iban" class="space-y-1.5">
          <Form.Control>
            {#snippet children({ props })}
              <Label class={sheetLabel}>IBAN</Label>
              <Input
                {...props}
                class={cn(sheetInput, "font-mono uppercase")}
                bind:value={$formData.iban}
                placeholder="UA..."
                oninput={(event) => {
                  $formData.iban = event.currentTarget.value.toUpperCase();
                }}
              />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>
      </div>
    </SheetSection>
  {/if}

  <SheetSection title="Додатково">
    <NotesDisclosure bind:expanded={notesOpen}>
      <Form.Field {form} name="notes" class="space-y-1.5">
        <Form.Control>
          {#snippet children({ props })}
            <Label class={sheetLabel}>Нотатки</Label>
            <Textarea
              {...props}
              class={sheetTextarea}
              bind:value={$formData.notes}
              placeholder="Додаткова інформація про клієнта..."
              rows={3}
            />
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>
    </NotesDisclosure>
  </SheetSection>
</SheetFrame>

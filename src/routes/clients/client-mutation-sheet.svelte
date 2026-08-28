<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import type { CountryCode } from "svelte-tel-input/types";
  import { validateTelInput } from "svelte-tel-input/validators";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import BriefcaseIcon from "@lucide/svelte/icons/briefcase";
  import BuildingIcon from "@lucide/svelte/icons/building-2";
  import CopyIcon from "@lucide/svelte/icons/copy";
  import CreditCardIcon from "@lucide/svelte/icons/credit-card";
  import FileTextIcon from "@lucide/svelte/icons/file-text";
  import UserIcon from "@lucide/svelte/icons/user";
  import UsersIcon from "@lucide/svelte/icons/users";

  import { Button } from "$lib/components/ui/button";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select/index.js";
  import { Textarea } from "$lib/components/ui/textarea";

  import MutationSheetShell from "$lib/components/mutation-sheet-shell.svelte";
  import PhoneInput from "$lib/components/phone-input.svelte";

  import type {
    ClientListItem,
    ClientPayload,
    ClientType,
  } from "$lib/bindings";

  import {
    createClientDetailsQuery,
    createClientsQuery,
    createCreateClientMutation,
    createUpdateClientMutation,
  } from "./queries";

  let {
    clientId = null,
    open = $bindable(false),
  }: { clientId?: number | null; open?: boolean } = $props();

  const formID = "client-mutation-form";
  const isEdit = $derived(clientId != null);

  const queryClient = useQueryClient();

  const createClientMutation = createCreateClientMutation(queryClient);
  const updateClientMutation = createUpdateClientMutation(queryClient);

  let clientsQuery = createClientsQuery();
  let clientsList = $derived(clientsQuery.data ?? []);
  let client = $derived(clientsList.find((c) => c.id === clientId) ?? null);

  let clientDetailsQuery = createClientDetailsQuery(() =>
    isEdit ? clientId : null,
  );
  let clientDetails = $derived(clientDetailsQuery.data?.details ?? null);

  let phoneCountry = $state<CountryCode>("UA");

  const TYPE_LABELS: Record<ClientType, string> = {
    individual: "Фізична особа",
    fop: "ФОП",
    company: "Організація",
  };

  const TYPE_ICONS = {
    individual: UserIcon,
    fop: BriefcaseIcon,
    company: BuildingIcon,
  } satisfies Record<ClientType, typeof CopyIcon>;

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

      // Client name: only the first name is required (for people)
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

      // If any part of a person's name is filled, the first name is required
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

      // Phone: required for individuals (the only type with a generic phone)
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

      // Duplicate phone across existing clients
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

      // Email format when present
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

      // Duplicate checks for unique identifiers (email is case-insensitive)
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

  /** True when another client (excluding the one being edited) has the value. */
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

  /** Контактна особа — сама ФОП: копіює ПІБ ФОП */
  function copyFopAsContact() {
    $formData.contactSurname = $formData.surname;
    $formData.contactFirstName = $formData.firstName;
    $formData.contactPatronymic = $formData.patronymic;
  }

  /** Контактна особа — керівник організації: копіює ПІБ керівника */
  function copyDirectorAsContact() {
    $formData.contactSurname = $formData.directorSurname;
    $formData.contactFirstName = $formData.directorFirstName;
    $formData.contactPatronymic = $formData.directorPatronymic;
  }

  const form = superForm(defaults(zod4(clientSchema)), {
    SPA: true,
    validators: zod4Client(clientSchema),
    validationMethod: "oninput",
    resetForm: false,
    id: formID,
    async onUpdate({ form: nativeForm }) {
      if (!nativeForm.valid) return;

      const payload = buildPayload(nativeForm.data);

      if (isEdit && clientId != null) {
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

  let didPrefill = $state(false);

  $effect(() => {
    if (!open) {
      didPrefill = false;
      return;
    }
    if (didPrefill) return;
    if (isEdit && (clientsQuery.isPending || clientDetailsQuery.isPending))
      return;

    didPrefill = true;

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
    } else {
      form.reset();
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

{#snippet sectionTitle(text: string, Icon?: typeof CopyIcon, hint?: string)}
  <div
    class="flex items-center gap-1.5 border-b border-border/70 pb-1 mt-4 first:mt-0"
  >
    {#if Icon}
      <Icon class="size-3.5 text-muted-foreground shrink-0" />
    {/if}
    <h3
      class="text-xs font-semibold uppercase tracking-wide text-muted-foreground"
    >
      {text}
    </h3>
  </div>
  {#if hint}
    <p class="text-xs text-muted-foreground">{hint}</p>
  {/if}
{/snippet}

{#snippet pibRow(
  surnameField: "surname" | "directorSurname" | "contactSurname",
  firstNameField: "firstName" | "directorFirstName" | "contactFirstName",
  patronymicField: "patronymic" | "directorPatronymic" | "contactPatronymic",
  requiredFirstName = false,
)}
  <div class="grid grid-cols-1 gap-2 sm:grid-cols-3">
    <Form.Field {form} name={surnameField}>
      <Form.Control>
        {#snippet children({ props })}
          <Label>Прізвище</Label>
          <Input {...props} bind:value={$formData[surnameField]} />
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>
    <Form.Field {form} name={firstNameField}>
      <Form.Control>
        {#snippet children({ props })}
          <Label>
            Ім'я{#if requiredFirstName}{@render requiredHint()}{/if}
          </Label>
          <Input {...props} bind:value={$formData[firstNameField]} />
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>
    <Form.Field {form} name={patronymicField}>
      <Form.Control>
        {#snippet children({ props })}
          <Label>По-батькові</Label>
          <Input {...props} bind:value={$formData[patronymicField]} />
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>
  </div>
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
  <Form.Field {form} name="clientType">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Тип клієнта</Label>
        <Select.Root type="single" bind:value={$formData.clientType} {...props}>
          <Select.Trigger class="w-full">
            {@const TypeIcon = TYPE_ICONS[$formData.clientType]}
            <span class="flex items-center gap-2">
              <TypeIcon class="size-4 text-muted-foreground" />
              {TYPE_LABELS[$formData.clientType]}
            </span>
          </Select.Trigger>
          <Select.Content>
            <Select.Item value="individual">
              <span class="flex items-center gap-2">
                <UserIcon class="size-4 text-muted-foreground" />
                {TYPE_LABELS["individual"]}
              </span>
            </Select.Item>
            <Select.Item value="fop">
              <span class="flex items-center gap-2">
                <BriefcaseIcon class="size-4 text-muted-foreground" />
                {TYPE_LABELS["fop"]}
              </span>
            </Select.Item>
            <Select.Item value="company">
              <span class="flex items-center gap-2">
                <BuildingIcon class="size-4 text-muted-foreground" />
                {TYPE_LABELS["company"]}
              </span>
            </Select.Item>
          </Select.Content>
        </Select.Root>
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  {#if $formData.clientType === "company"}
    <Form.Field {form} name="name">
      <Form.Control>
        {#snippet children({ props })}
          <Label>Коротка назва{@render requiredHint()}</Label>
          <Input
            {...props}
            bind:value={$formData.name}
            placeholder="Автосервіс Плюс"
          />
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>
  {:else}
    {@render pibRow("surname", "firstName", "patronymic", true)}
  {/if}

  {#if $formData.clientType === "individual"}
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

    <Form.Field {form} name="email">
      <Form.Control>
        {#snippet children({ props })}
          <Label>Email</Label>
          <Input
            {...props}
            bind:value={$formData.email}
            type="email"
            placeholder="email@example.com"
          />
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>
  {/if}

  {#if $formData.clientType === "company"}
    {@render sectionTitle("Дані організації", BuildingIcon)}

    <Form.Field {form} name="legalName">
      <Form.Control>
        {#snippet children({ props })}
          <Label>Юридична назва{@render requiredHint()}</Label>
          <Input
            {...props}
            bind:value={$formData.legalName}
            placeholder="ТОВ «Автосервіс Плюс»"
          />
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>

    <Form.Field {form} name="edrpou">
      <Form.Control>
        {#snippet children({ props })}
          <Label>ЄДРПОУ{@render requiredHint()}</Label>
          <Input
            {...props}
            bind:value={$formData.edrpou}
            inputmode="numeric"
            placeholder="12345678"
          />
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>

    <Form.Field {form} name="legalAddress">
      <Form.Control>
        {#snippet children({ props })}
          <Label>Юридична адреса{@render requiredHint()}</Label>
          <Textarea
            {...props}
            bind:value={$formData.legalAddress}
            placeholder="Адреса реєстрації організації..."
            rows={2}
          />
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>

    <Form.Field {form} name="vatPayer">
      <Form.Control>
        {#snippet children({ props })}
          <div class="flex items-center gap-2">
            <Checkbox
              {...props}
              id="vatPayer"
              bind:checked={$formData.vatPayer}
            />
            <Label for="vatPayer" class="font-normal">Платник ПДВ</Label>
          </div>
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>

    {@render sectionTitle("Керівник", UserIcon)}

    <div class="rounded-lg border border-border bg-muted/30 p-3">
      {@render pibRow(
        "directorSurname",
        "directorFirstName",
        "directorPatronymic",
      )}
    </div>
  {:else if $formData.clientType === "fop"}
    {@render sectionTitle("Дані ФОП", BriefcaseIcon)}

    <Form.Field {form} name="rnokpp">
      <Form.Control>
        {#snippet children({ props })}
          <Label>РНОКПП{@render requiredHint()}</Label>
          <Input
            {...props}
            bind:value={$formData.rnokpp}
            inputmode="numeric"
            placeholder="1234567890"
          />
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>

    <Form.Field {form} name="legalAddress">
      <Form.Control>
        {#snippet children({ props })}
          <Label>Адреса реєстрації</Label>
          <Textarea
            {...props}
            bind:value={$formData.legalAddress}
            placeholder="Адреса реєстрації ФОП..."
            rows={2}
          />
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>
  {/if}

  {#if $formData.clientType !== "individual"}
    {@render sectionTitle("Контактна особа", UsersIcon)}

    <div class="rounded-lg border border-border bg-muted/30 p-3 space-y-3">
      <Button
        variant="outline"
        size="sm"
        class="w-full text-xs"
        type="button"
        onclick={$formData.clientType === "company"
          ? copyDirectorAsContact
          : copyFopAsContact}
      >
        <CopyIcon class="size-3.5 mr-1" />
        {$formData.clientType === "company"
          ? "Копіювати від керівника"
          : "Копіювати з ФОП"}
      </Button>

      {@render pibRow(
        "contactSurname",
        "contactFirstName",
        "contactPatronymic",
      )}

      <Form.Field {form} name="contactPhone">
        <Form.Control>
          {#snippet children({ props })}
            <Label>Телефон контактної особи</Label>
            <PhoneInput
              {...props}
              bind:value={$formData.contactPhone}
              bind:country={phoneCountry}
            />
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>

      <Form.Field {form} name="contactEmail">
        <Form.Control>
          {#snippet children({ props })}
            <Label>Email контактної особи</Label>
            <Input
              {...props}
              bind:value={$formData.contactEmail}
              type="email"
              placeholder="email@example.com"
            />
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>
    </div>

    {@render sectionTitle("Реквізити", CreditCardIcon)}

    <div class="rounded-lg border border-border bg-muted/30 p-3 space-y-3">
      <Form.Field {form} name="bankName">
        <Form.Control>
          {#snippet children({ props })}
            <Label>Назва банку</Label>
            <Input {...props} bind:value={$formData.bankName} />
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>

      <Form.Field {form} name="iban">
        <Form.Control>
          {#snippet children({ props })}
            <Label>IBAN</Label>
            <Input {...props} bind:value={$formData.iban} placeholder="UA..." />
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>
    </div>
  {/if}

  {@render sectionTitle("Нотатки", FileTextIcon)}

  <Form.Field {form} name="notes">
    <Form.Control>
      {#snippet children({ props })}
        <Textarea
          {...props}
          bind:value={$formData.notes}
          placeholder="Додаткова інформація про клієнта..."
          rows={3}
        />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>
</MutationSheetShell>

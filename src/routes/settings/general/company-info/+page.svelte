<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import { beforeNavigate, goto } from "$app/navigation";

  import ArrowLeftIcon from "@lucide/svelte/icons/arrow-left";
  import BriefcaseIcon from "@lucide/svelte/icons/briefcase";
  import BuildingIcon from "@lucide/svelte/icons/building-2";
  import CheckIcon from "@lucide/svelte/icons/check";
  import CreditCardIcon from "@lucide/svelte/icons/credit-card";
  import MapPinIcon from "@lucide/svelte/icons/map-pin";
  import UserIcon from "@lucide/svelte/icons/user";

  import { Button } from "$lib/components/ui/button";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select/index.js";
  import Spinner from "$lib/components/ui/spinner/spinner.svelte";
  import { Textarea } from "$lib/components/ui/textarea";

  import PageLayout from "$lib/components/page-layout.svelte";

  import type { CompanyInfo, CompanyKind } from "$lib/bindings";

  import { createUpdateCompanyInfoMutation } from "../../queries";
  import {
    getSettingsContext,
    SETTINGS_CONTENT_WIDTH_CLASS,
  } from "../../settings-context";

  const queryClient = useQueryClient();
  const { draft } = getSettingsContext();

  const updateCompanyInfoMutation =
    createUpdateCompanyInfoMutation(queryClient);

  // ---------- Form ----------

  const KIND_LABELS: Record<CompanyKind, string> = {
    fop: "ФОП",
    company: "Організація",
  };

  const KIND_ICONS = {
    fop: BriefcaseIcon,
    company: BuildingIcon,
  } satisfies Record<CompanyKind, typeof CheckIcon>;

  // All fields are optional; only the format is checked when a field has
  // content. Errors are shown per field and block the save until fixed.
  const companyInfoSchema = z
    .object({
      kind: z.enum(["fop", "company"]).default("fop"),
      name: z.string().default(""),
      legalName: z.string().default(""),
      legalAddress: z.string().default(""),
      physicalAddress: z.string().default(""),
      edrpou: z.string().default(""),
      rnokpp: z.string().default(""),
      surname: z.string().default(""),
      firstName: z.string().default(""),
      patronymic: z.string().default(""),
      directorSurname: z.string().default(""),
      directorFirstName: z.string().default(""),
      directorPatronymic: z.string().default(""),
      iban: z.string().default(""),
      bankName: z.string().default(""),
      vatPayer: z.boolean().default(false),
    })
    .superRefine((data, ctx) => {
      const iban = data.iban.trim();
      if (iban !== "" && !/^[A-Z]{2}\d{2}[A-Z0-9]{11,30}$/.test(iban)) {
        ctx.addIssue({
          code: "custom",
          path: ["iban"],
          message: "Неправильний IBAN",
        });
      }

      const rnokpp = data.rnokpp.trim();
      if (rnokpp !== "" && !/^\d{10}$/.test(rnokpp)) {
        ctx.addIssue({
          code: "custom",
          path: ["rnokpp"],
          message: "РНОКПП — 10 цифр",
        });
      }

      const edrpou = data.edrpou.trim();
      if (edrpou !== "" && !/^\d{8}$/.test(edrpou)) {
        ctx.addIssue({
          code: "custom",
          path: ["edrpou"],
          message: "ЄДРПОУ — 8 цифр",
        });
      }
    });

  type CompanyInfoFormData = z.infer<typeof companyInfoSchema>;
  type TextField = Exclude<keyof CompanyInfoFormData, "kind" | "vatPayer">;

  function trimToNull(value: string): string | null {
    const trimmed = value.trim();
    return trimmed === "" ? null : trimmed;
  }

  function buildPayload(data: CompanyInfoFormData): CompanyInfo {
    return {
      kind: data.kind,
      name: trimToNull(data.name),
      legalName: trimToNull(data.legalName),
      legalAddress: trimToNull(data.legalAddress),
      physicalAddress: trimToNull(data.physicalAddress),
      edrpou: trimToNull(data.edrpou),
      rnokpp: trimToNull(data.rnokpp),
      surname: trimToNull(data.surname),
      firstName: trimToNull(data.firstName),
      patronymic: trimToNull(data.patronymic),
      directorSurname: trimToNull(data.directorSurname),
      directorFirstName: trimToNull(data.directorFirstName),
      directorPatronymic: trimToNull(data.directorPatronymic),
      iban: trimToNull(data.iban),
      bankName: trimToNull(data.bankName),
      vatPayer: data.vatPayer,
      // Placeholder: the backend recomputes this flag from the actual values
      isComplete: false,
    };
  }

  function fillForm(saved: CompanyInfo | null) {
    $formData.kind = saved?.kind ?? "fop";
    $formData.name = saved?.name ?? "";
    $formData.legalName = saved?.legalName ?? "";
    $formData.legalAddress = saved?.legalAddress ?? "";
    $formData.physicalAddress = saved?.physicalAddress ?? "";
    $formData.edrpou = saved?.edrpou ?? "";
    $formData.rnokpp = saved?.rnokpp ?? "";
    $formData.surname = saved?.surname ?? "";
    $formData.firstName = saved?.firstName ?? "";
    $formData.patronymic = saved?.patronymic ?? "";
    $formData.directorSurname = saved?.directorSurname ?? "";
    $formData.directorFirstName = saved?.directorFirstName ?? "";
    $formData.directorPatronymic = saved?.directorPatronymic ?? "";
    $formData.iban = saved?.iban ?? "";
    $formData.bankName = saved?.bankName ?? "";
    $formData.vatPayer = saved?.vatPayer ?? false;
  }

  const form = superForm(defaults(zod4(companyInfoSchema)), {
    SPA: true,
    validators: zod4Client(companyInfoSchema),
    validationMethod: "oninput",
    resetForm: false,
    id: "company-info-form",
    async onUpdate({ form: nativeForm, cancel }) {
      if (!nativeForm.valid) return;

      const payload = buildPayload(nativeForm.data);
      const saved = await updateCompanyInfoMutation
        .mutateAsync(payload)
        .catch(() => null);
      if (saved && draft) {
        draft.companyInfo = saved;
        savedPayloadKey = JSON.stringify(payload);
      }
      cancel();
    },
  });

  const { form: formData, enhance, submitting } = form;

  /** JSON key of the last persisted payload (or the one the form was loaded with). */
  let savedPayloadKey = $state<string | null>(null);

  const hasChanges = $derived(
    savedPayloadKey !== null &&
      JSON.stringify(buildPayload($formData)) !== savedPayloadKey,
  );

  /** Discard unsaved edits and restore the last saved values. */
  function discard() {
    const saved = draft?.companyInfo ?? null;
    if (saved) {
      fillForm(saved);
    } else {
      form.reset();
    }
  }

  // ---------- Unsaved-changes warning ----------

  let confirmLeaveOpen = $state(false);
  let pendingLeaveUrl: string | null = null;
  let allowLeave = false;

  beforeNavigate((nav) => {
    if (allowLeave) {
      allowLeave = false;
      return;
    }
    if (!hasChanges) return;
    nav.cancel();
    pendingLeaveUrl = nav.to?.url.toString() ?? null;
    confirmLeaveOpen = true;
  });

  function confirmLeave() {
    allowLeave = true;
    confirmLeaveOpen = false;
    if (pendingLeaveUrl) {
      goto(pendingLeaveUrl);
    }
  }

  // ---------- Prefill ----------

  let didPrefill = $state(false);

  $effect(() => {
    if (didPrefill || !draft) return;
    didPrefill = true;
    fillForm(draft.companyInfo ?? null);
    savedPayloadKey = JSON.stringify(buildPayload($formData));
  });
</script>

{#snippet sectionTitle(text: string, Icon?: typeof CheckIcon)}
  <div class="flex items-center gap-1.5 border-b border-border/70 pb-1">
    {#if Icon}
      <Icon class="size-3.5 text-muted-foreground shrink-0" />
    {/if}
    <h3
      class="text-xs font-semibold uppercase tracking-wide text-muted-foreground"
    >
      {text}
    </h3>
  </div>
{/snippet}

{#snippet inputField(
  field: TextField,
  label: string,
  placeholder = "",
  inputmode: "numeric" | "decimal" | undefined = undefined,
)}
  <Form.Field {form} name={field}>
    <Form.Control>
      {#snippet children({ props })}
        <Label>{label}</Label>
        <Input
          {...props}
          bind:value={$formData[field]}
          placeholder={placeholder || undefined}
          {inputmode}
        />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>
{/snippet}

{#snippet textAreaField(field: TextField, label: string, placeholder = "")}
  <Form.Field {form} name={field}>
    <Form.Control>
      {#snippet children({ props })}
        <Label>{label}</Label>
        <Textarea
          {...props}
          bind:value={$formData[field]}
          placeholder={placeholder || undefined}
          rows={2}
        />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>
{/snippet}

{#snippet pibRow(
  surnameField: "surname" | "directorSurname",
  firstNameField: "firstName" | "directorFirstName",
  patronymicField: "patronymic" | "directorPatronymic",
)}
  <div class="grid grid-cols-1 gap-2 sm:grid-cols-3">
    {@render inputField(surnameField, "Прізвище")}
    {@render inputField(firstNameField, "Ім'я")}
    {@render inputField(patronymicField, "По-батькові")}
  </div>
{/snippet}

<PageLayout contentWidthClass={SETTINGS_CONTENT_WIDTH_CLASS}>
  <div class="mt-4 flex items-center justify-between gap-3">
    <div class="flex items-center gap-3">
      <Button
        variant="ghost"
        size="icon-lg"
        aria-label="Назад до загальних налаштувань"
        onclick={() => goto("/settings/general")}
      >
        <ArrowLeftIcon class="size-4" />
      </Button>
      <h1 class="font-medium">Дані бізнесу</h1>
    </div>
  </div>

  <div class="mt-4">
    <div
      id="settings-company-info"
      class="border border-border rounded bg-card"
    >
      <form use:enhance id="company-info-form" class="flex flex-col gap-4 p-4">
        <div
          class="sticky top-0 z-10 -mx-4 -mt-4 flex flex-col gap-4 rounded-t border-b border-border/70 bg-card px-4 pb-3 pt-4"
        >
          <div class="flex flex-wrap items-center justify-between gap-3">
            <Form.Field {form} name="kind" class="w-full sm:w-64">
              <Form.Control>
                {#snippet children({ props })}
                  <Label>Форма ведення бізнесу</Label>
                  <Select.Root
                    type="single"
                    bind:value={$formData.kind}
                    {...props}
                  >
                    <Select.Trigger class="w-full">
                      {@const KindIcon = KIND_ICONS[$formData.kind]}
                      <span class="flex items-center gap-2">
                        <KindIcon class="size-4 text-muted-foreground" />
                        {KIND_LABELS[$formData.kind]}
                      </span>
                    </Select.Trigger>
                    <Select.Content>
                      <Select.Item value="fop">
                        <span class="flex items-center gap-2">
                          <BriefcaseIcon class="size-4 text-muted-foreground" />
                          {KIND_LABELS["fop"]}
                        </span>
                      </Select.Item>
                      <Select.Item value="company">
                        <span class="flex items-center gap-2">
                          <BuildingIcon class="size-4 text-muted-foreground" />
                          {KIND_LABELS["company"]}
                        </span>
                      </Select.Item>
                    </Select.Content>
                  </Select.Root>
                {/snippet}
              </Form.Control>
              <Form.FieldErrors />
            </Form.Field>

            <div class="flex items-center gap-2">
              <Button
                variant="outline"
                type="button"
                disabled={!hasChanges || $submitting}
                onclick={discard}
              >
                Скасувати
              </Button>
              <Button type="submit" disabled={!hasChanges || $submitting}>
                {#if $submitting}Збереження <Spinner />{:else}Зберегти{/if}
              </Button>
            </div>
          </div>

          {#if updateCompanyInfoMutation.isError}
            <div class="flex items-center justify-end gap-3">
              <p class="text-xs text-destructive">
                Не вдалося зберегти:
                {updateCompanyInfoMutation.error?.message}
              </p>
            </div>
          {/if}
        </div>

        {@render sectionTitle("Основні дані", MapPinIcon)}

        {@render textAreaField(
          "physicalAddress",
          "Фактична адреса",
          "Фізична адреса, де фактично знаходиться бізнес...",
        )}

        <Form.Field {form} name="vatPayer">
          <Form.Control>
            {#snippet children({ props })}
              <div class="flex items-center gap-2">
                <Checkbox
                  {...props}
                  id="companyInfoVatPayer"
                  bind:checked={$formData.vatPayer}
                />
                <Label for="companyInfoVatPayer" class="font-normal"
                  >Платник ПДВ</Label
                >
              </div>
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>

        {#if $formData.kind === "company"}
          {@render sectionTitle("Дані організації", BuildingIcon)}

          {@render inputField("name", "Коротка назва", "Автосервіс Плюс")}
          {@render inputField(
            "legalName",
            "Юридична назва",
            "ТОВ «Автосервіс Плюс»",
          )}
          {@render inputField("edrpou", "ЄДРПОУ", "12345678", "numeric")}
          {@render textAreaField(
            "legalAddress",
            "Юридична адреса",
            "Адреса реєстрації організації...",
          )}

          {@render sectionTitle("Керівник", UserIcon)}

          {@render pibRow(
            "directorSurname",
            "directorFirstName",
            "directorPatronymic",
          )}
        {:else}
          {@render sectionTitle("Дані ФОП", BriefcaseIcon)}

          {@render pibRow("surname", "firstName", "patronymic")}

          {@render inputField("rnokpp", "РНОКПП", "1234567890", "numeric")}
          {@render textAreaField(
            "legalAddress",
            "Адреса реєстрації",
            "Адреса реєстрації ФОП...",
          )}
        {/if}

        {@render sectionTitle("Банківські реквізити", CreditCardIcon)}

        {@render inputField("bankName", "Назва банку")}
        {@render inputField("iban", "IBAN", "UA...")}
      </form>
    </div>
  </div>
</PageLayout>

<Dialog.Root bind:open={confirmLeaveOpen}>
  <Dialog.Content class="sm:max-w-sm">
    <Dialog.Header>
      <Dialog.Title>Незбережені зміни</Dialog.Title>
      <Dialog.Description>
        У вас є незбережені зміни. Вийти без збереження?
      </Dialog.Description>
    </Dialog.Header>
    <Dialog.Footer>
      <Button variant="outline" onclick={() => (confirmLeaveOpen = false)}>
        Залишитися
      </Button>
      <Button variant="warning" onclick={confirmLeave}>
        Вийти без збереження
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

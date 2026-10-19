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
  import PercentIcon from "@lucide/svelte/icons/percent";
  import UserIcon from "@lucide/svelte/icons/user";

  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import * as Item from "$lib/components/ui/item/index.js";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select/index.js";
  import Spinner from "$lib/components/ui/spinner/spinner.svelte";
  import { Switch } from "$lib/components/ui/switch";
  import { Textarea } from "$lib/components/ui/textarea";

  import MutationSheetSectionTitle from "$lib/components/mutation-sheet-section-title.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import type { CompanyInfo, CompanyKind } from "$lib/bindings";

  import {
    createUpdateCompanyInfoMutation,
    createUpdateDefaultPricesIncludeVatMutation,
    createUpdateDefaultVatRateMutation,
  } from "../../queries";
  import {
    getSettingsContext,
    SETTINGS_CONTENT_WIDTH_CLASS,
  } from "../../settings-context";

  const queryClient = useQueryClient();
  const { draft } = getSettingsContext();

  const updateCompanyInfoMutation =
    createUpdateCompanyInfoMutation(queryClient);
  const updateVatPayerMutation = createUpdateCompanyInfoMutation(queryClient);
  const updateVatRateMutation = createUpdateDefaultVatRateMutation(queryClient);
  const updatePricesIncludeVatMutation =
    createUpdateDefaultPricesIncludeVatMutation(queryClient);

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
      phone: z.string().default(""),
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
  type TextField = Exclude<keyof CompanyInfoFormData, "kind">;

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
      phone: trimToNull(data.phone),
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
      vatPayer: draft?.companyInfo?.vatPayer ?? false,
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
    $formData.phone = saved?.phone ?? "";
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
  }

  const form = superForm(defaults(zod4(companyInfoSchema)), {
    SPA: true,
    validators: zod4Client(companyInfoSchema),
    validationMethod: "oninput",
    resetForm: false,
    id: "company-info-form",
    async onUpdate({ form: nativeForm, cancel }) {
      if (!nativeForm.valid) return;
      if (updateVatPayerMutation.isPending) {
        cancel();
        return;
      }

      const payload = buildPayload(nativeForm.data);

      try {
        const saved = await updateCompanyInfoMutation.mutateAsync(payload);
        if (saved && draft) {
          draft.companyInfo = saved;
          savedPayloadKey = JSON.stringify(payload);
        }
      } catch {
        // The mutation exposes its error in the UI.
      } finally {
        cancel();
      }
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

  // ---------- Tax settings (auto-saved) ----------

  function normalizeVatRate(value: string): string {
    return value.trim().replace(",", ".");
  }

  let vatRateInput = $state(draft?.defaultVatRate ?? "20");
  let vatRateFocused = $state(false);

  $effect(() => {
    if (!vatRateFocused && draft) {
      vatRateInput = draft.defaultVatRate;
    }
  });

  async function commitVatRate(raw: string) {
    if (!draft) return;
    const normalized = normalizeVatRate(raw);
    if (!/^\d{1,3}(\.\d{1,2})?$/.test(normalized) || Number(normalized) > 100) {
      vatRateInput = draft.defaultVatRate;
      return;
    }

    const previous = draft.defaultVatRate;
    draft.defaultVatRate = normalized;
    vatRateInput = normalized;

    try {
      const saved = await updateVatRateMutation.mutateAsync(normalized);
      draft.defaultVatRate = saved;
      vatRateInput = saved;
    } catch {
      draft.defaultVatRate = previous;
      vatRateInput = previous;
    }
  }

  function onVatRateKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      (e.currentTarget as HTMLInputElement).blur();
    }
  }

  /** Keep the form's saved-payload baseline in step after an auto-saved payer change. */
  function withVatPayer(key: string, vatPayer: boolean): string {
    const payload = JSON.parse(key) as CompanyInfo;
    return JSON.stringify({ ...payload, vatPayer });
  }

  async function setVatPayer(next: boolean) {
    const current = draft?.companyInfo;
    if (
      !draft ||
      !current ||
      $submitting ||
      updateCompanyInfoMutation.isPending
    )
      return;

    const previous = current;
    const updated: CompanyInfo = {
      ...current,
      vatPayer: next,
    };
    draft.companyInfo = updated;

    try {
      const saved = await updateVatPayerMutation.mutateAsync(updated);
      if (!saved) {
        draft.companyInfo = previous;
        return;
      }
      draft.companyInfo = saved;
      if (savedPayloadKey !== null) {
        savedPayloadKey = withVatPayer(savedPayloadKey, saved.vatPayer);
      }
    } catch {
      draft.companyInfo = previous;
    }
  }

  async function setPricesIncludeVat(next: boolean) {
    if (!draft) return;
    const previous = draft.defaultPricesIncludeVat;
    draft.defaultPricesIncludeVat = next;

    try {
      const saved = await updatePricesIncludeVatMutation.mutateAsync(next);
      draft.defaultPricesIncludeVat = saved;
    } catch {
      draft.defaultPricesIncludeVat = previous;
    }
  }
</script>

{#snippet inputField(
  field: TextField,
  label: string,
  placeholder = "",
  inputmode: "numeric" | "decimal" | undefined = undefined,
  fieldClass = "",
)}
  <Form.Field {form} name={field} class={fieldClass}>
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

{#snippet textAreaField(
  field: TextField,
  label: string,
  placeholder = "",
  fieldClass = "",
)}
  <Form.Field {form} name={field} class={fieldClass}>
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
  <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
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

  <form use:enhance id="company-info-form" class="mt-4 flex flex-col gap-4">
    <div
      id="settings-company-info"
      class="border border-border rounded bg-card"
    >
      <div
        class="sticky top-0 z-10 flex flex-col gap-4 rounded-t border-b border-border/70 bg-card px-4 pb-3 pt-4"
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
              disabled={!hasChanges ||
                $submitting ||
                updateVatPayerMutation.isPending}
              onclick={discard}
            >
              Скасувати
            </Button>
            <Button
              type="submit"
              disabled={!hasChanges ||
                $submitting ||
                updateVatPayerMutation.isPending}
            >
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

      <div class="flex flex-col gap-4 p-4">
        <MutationSheetSectionTitle
          text="Основні дані"
          icon={MapPinIcon}
          class="mt-0"
        />

        <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
          {@render inputField(
            "name",
            "Назва бізнесу",
            "Автосервіс Плюс",
            undefined,
            "sm:col-span-2",
          )}
          {@render inputField("phone", "Телефон", "+380...")}
        </div>

        <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
          {@render textAreaField(
            "physicalAddress",
            "Фактична адреса",
            "Фізична адреса, де фактично знаходиться бізнес...",
            "sm:col-span-2",
          )}
        </div>

        {#if $formData.kind === "company"}
          <MutationSheetSectionTitle
            text="Дані організації"
            icon={BuildingIcon}
            class="mt-0"
          />

          <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
            {@render inputField(
              "legalName",
              "Юридична назва",
              "ТОВ «Автосервіс Плюс»",
              undefined,
              "sm:col-span-2",
            )}
            {@render inputField("edrpou", "ЄДРПОУ", "12345678", "numeric")}
          </div>
          <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
            {@render textAreaField(
              "legalAddress",
              "Юридична адреса",
              "Адреса реєстрації організації...",
              "sm:col-span-2",
            )}
          </div>

          <MutationSheetSectionTitle
            text="Керівник"
            icon={UserIcon}
            class="mt-0"
          />

          {@render pibRow(
            "directorSurname",
            "directorFirstName",
            "directorPatronymic",
          )}
        {:else}
          <MutationSheetSectionTitle
            text="Дані ФОП"
            icon={BriefcaseIcon}
            class="mt-0"
          />

          {@render pibRow("surname", "firstName", "patronymic")}

          <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
            {@render inputField("rnokpp", "РНОКПП", "1234567890", "numeric")}
            {@render textAreaField(
              "legalAddress",
              "Адреса реєстрації",
              "Адреса реєстрації ФОП...",
              "sm:col-span-2",
            )}
          </div>
        {/if}

        <MutationSheetSectionTitle
          text="Банківські реквізити"
          icon={CreditCardIcon}
          class="mt-0"
        />

        <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
          {@render inputField(
            "bankName",
            "Назва банку",
            "",
            undefined,
            "sm:col-span-2",
          )}
          {@render inputField("iban", "IBAN", "UA...")}
        </div>
      </div>
    </div>
  </form>

  <div
    id="settings-company-tax"
    class="mt-4 border border-border rounded bg-card"
  >
    <div class="px-4 pt-3 pb-1">
      <MutationSheetSectionTitle
        text="Оподаткування"
        icon={PercentIcon}
        class="mt-0"
      />
    </div>

    <Item.Root id="settings-vat-payer">
      <Item.Content>
        <Item.Title>Платник ПДВ</Item.Title>
        <Item.Description
          >Підприємство зареєстроване як платник ПДВ</Item.Description
        >
      </Item.Content>
      <Item.Actions>
        <Switch
          checked={draft?.companyInfo?.vatPayer ?? false}
          onCheckedChange={setVatPayer}
          disabled={!draft?.companyInfo ||
            updateVatPayerMutation.isPending ||
            updateCompanyInfoMutation.isPending ||
            $submitting}
          aria-label="Платник ПДВ"
        />
      </Item.Actions>
    </Item.Root>
    {#if updateVatPayerMutation.isError}
      <p class="px-4 pb-2 text-xs text-destructive">
        Не вдалося зберегти:
        {updateVatPayerMutation.error?.message}
      </p>
    {/if}

    <Item.Root id="settings-vat-rate">
      <Item.Content>
        <Item.Title>Ставка ПДВ</Item.Title>
        <Item.Description
          >Ставка за замовчуванням для нових замовлень</Item.Description
        >
      </Item.Content>
      <Item.Actions>
        <div class="flex items-center gap-2">
          <Input
            bind:value={vatRateInput}
            type="text"
            inputmode="decimal"
            aria-label="Ставка ПДВ"
            class="w-20 text-right"
            disabled={updateVatRateMutation.isPending}
            onfocus={() => (vatRateFocused = true)}
            onblur={() => {
              vatRateFocused = false;
              commitVatRate(vatRateInput);
            }}
            onkeydown={onVatRateKeydown}
          />
          <p>%</p>
        </div>
      </Item.Actions>
    </Item.Root>
    {#if updateVatRateMutation.isError}
      <p class="px-4 pb-2 text-xs text-destructive">
        Не вдалося зберегти ставку:
        {updateVatRateMutation.error?.message}
      </p>
    {/if}

    <Item.Root id="settings-vat-inclusive">
      <Item.Content>
        <Item.Title>Ціни включають ПДВ</Item.Title>
        <Item.Description
          >Вважати ціни в нових замовленнях такими, що вже включають ПДВ, за
          замовчуванням</Item.Description
        >
      </Item.Content>
      <Item.Actions>
        <Switch
          checked={draft?.defaultPricesIncludeVat ?? true}
          onCheckedChange={setPricesIncludeVat}
          disabled={updatePricesIncludeVatMutation.isPending}
          aria-label="Ціни включають ПДВ"
        />
      </Item.Actions>
    </Item.Root>
    {#if updatePricesIncludeVatMutation.isError}
      <p class="px-4 pb-2 text-xs text-destructive">
        Не вдалося зберегти:
        {updatePricesIncludeVatMutation.error?.message}
      </p>
    {/if}
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

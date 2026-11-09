<script lang="ts">
  import { createQueries, useQueryClient } from "@tanstack/svelte-query";
  import { tick } from "svelte";
  import { toast } from "svelte-sonner";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import UserIcon from "@lucide/svelte/icons/user";

  import { Button } from "$lib/components/ui/button";
  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Textarea } from "$lib/components/ui/textarea";

  import ComboSelect from "$lib/components/combo-select.svelte";
  import ClientPicker from "$lib/components/sheet/client-picker.svelte";
  import {
    sheetControlHeight,
    sheetInput,
    sheetLabel,
    sheetSecondaryButton,
    sheetTextarea,
  } from "$lib/components/sheet/field-styles";
  import NotesDisclosure from "$lib/components/sheet/notes-disclosure.svelte";
  import OwnerCard from "$lib/components/sheet/owner-card.svelte";
  import PlateTypePicker from "$lib/components/sheet/plate-type-picker.svelte";
  import { interpretQuery } from "$lib/components/sheet/query-prefill";
  import RequiredMark from "$lib/components/sheet/required-mark.svelte";
  import SegmentedControl from "$lib/components/sheet/segmented-control.svelte";
  import SheetFrame from "$lib/components/sheet/sheet-frame.svelte";
  import SheetSection from "$lib/components/sheet/sheet-section.svelte";
  import SuggestInput from "$lib/components/suggest-input.svelte";

  import type { DriveType, FuelType, PlateType } from "$lib/bindings";
  import { brandSearchValue } from "$lib/brands";
  import { normalizePlateText, plateTypeMeta } from "$lib/plate";
  import { createRecentItemsQuery } from "$lib/recent-items";
  import { cn } from "$lib/utils";

  import { clientsQueryOptions } from "../clients/queries";
  import { driveTypeLabels, fuelTypeLabels } from "./enum-labels";
  import {
    carBrandsQueryOptions,
    carsListQueryOptions,
    createCarModelsQuery,
    createCreateCarBrandMutation,
    createCreateCarMutation,
    createUpdateCarMutation,
  } from "./queries";

  let {
    active = true,
    carId = null,
    ownerClientId = null,
    ownerName = "",
    initialQuery = "",
    onCancel,
    onCreated,
    onBack,
    backLabel,
    stepLabel,
  }: {
    active?: boolean;
    carId?: number | null;
    ownerClientId?: number | null;
    ownerName?: string;
    initialQuery?: string;
    onCancel: () => void;
    onCreated: (id: number) => void;
    onBack?: () => void;
    backLabel?: string;
    stepLabel?: string;
  } = $props();

  const formID = "car-sheet-form";
  const isEdit = $derived(carId != null);
  const ownerIsLocked = $derived(!isEdit && ownerClientId != null);

  const queryClient = useQueryClient();
  const createCarMutation = createCreateCarMutation(queryClient);
  const updateCarMutation = createUpdateCarMutation(queryClient);
  const createCarBrandMutation = createCreateCarBrandMutation(queryClient);

  const combinedQueries = createQueries(() => ({
    queries: [
      carsListQueryOptions(),
      clientsQueryOptions(),
      carBrandsQueryOptions(),
    ],
    combine: (results) => ({
      cars: results[0],
      clients: results[1],
      carBrands: results[2],
      isReady: results.every((result) => result.isSuccess),
      isLoading: results.some((result) => result.isPending),
    }),
  }));

  const carsList = $derived(combinedQueries.cars.data ?? []);
  const clients = $derived(combinedQueries.clients.data ?? []);
  const carDto = $derived(carsList.find((dto) => dto.car.id === carId) ?? null);
  const brandList = $derived(combinedQueries.carBrands.data ?? []);

  const VIN_RE = /^[A-HJ-NPR-Z0-9]{17}$/;

  function normalizeVin(value: string) {
    return value.toUpperCase().replace(/[\s-]/g, "");
  }

  const currentYear = new Date().getFullYear();

  const yearSchema = z
    .string()
    .default("")
    .superRefine((value, ctx) => {
      const text = value.trim();
      if (text === "") {
        ctx.addIssue({ code: "custom", message: "Обов'язкове поле" });
        return;
      }
      if (!/^\d{4}$/.test(text)) {
        ctx.addIssue({ code: "custom", message: "Неправильний рік" });
        return;
      }
      const year = Number(text);
      if (year < 1950) {
        ctx.addIssue({
          code: "custom",
          message: "Рік не може бути раніше 1950",
        });
      } else if (year > currentYear + 1) {
        ctx.addIssue({
          code: "custom",
          message: "Рік не може бути в далекому майбутньому",
        });
      }
    });

  const carSchema = z.object({
    clientId: z
      .number({ error: "Обов'язкове поле" })
      .min(0, { error: "Обов'язкове поле" })
      .default(-1),

    brandId: z
      .number({ error: "Обов'язкове поле" })
      .positive({ error: "Обов'язкове поле" })
      .default(-1),

    model: z.string().min(1, { error: "Обов'язкове поле" }),
    year: yearSchema,

    licensePlate: z.string().nullable().default(null),

    plateType: z
      .enum(["ua", "ua_old", "military", "temporary", "foreign", "other"])
      .default("ua"),
    plateCountry: z
      .string()
      .nullable()
      .default(null)
      .transform((val) => (val ? val.trim().toUpperCase() : null)),

    vin: z
      .string()
      .nullable()
      .default(null)
      .transform((val) => (val ? normalizeVin(val) : null))
      .transform((val) => (val === "" ? null : val)),

    engineCode: z
      .string()
      .trim()
      .nullable()
      .default(null)
      .transform((val) => (val === "" ? null : val)),

    engineDisplacement: z
      .number()
      .positive({ error: "Об'єм двигуна має бути більшим за нуль" })
      .nullable()
      .default(null),

    fuelType: z
      .enum(["Gasoline", "Diesel", "LpgCng", "Hybrid", "Electric"])
      .nullable()
      .default(null),

    driveType: z
      .enum(["FrontWheel", "RearWheel", "AllWheel", "FourWheel"])
      .nullable()
      .default(null),

    notes: z
      .string()
      .nullable()
      .default(null)
      .transform((val) => (val === "" ? null : val)),
  });

  const extendedSchema = carSchema
    .refine(
      (data) => {
        if (!data.licensePlate) return true;
        const duplicateCar = carsList.find(
          (dto) =>
            normalizePlateText(dto.car.licensePlate) ===
            normalizePlateText(data.licensePlate),
        );
        if (!duplicateCar) return true;
        if (isEdit && duplicateCar.car.id === carDto?.car.id) return true;
        return false;
      },
      {
        message: "Автомобіль з таким реєстраційним номером вже існує",
        path: ["licensePlate"],
      },
    )
    .refine(
      (data) => data.plateType !== "foreign" || data.plateCountry != null,
      {
        message: "Оберіть країну для іноземного номера",
        path: ["plateCountry"],
      },
    )
    .refine(
      (data) => {
        if (!data.vin) return true;
        const duplicateCar = carsList.find((dto) => dto.car.vin === data.vin);
        if (!duplicateCar) return true;
        if (isEdit && duplicateCar.car.id === carDto?.car.id) return true;
        return false;
      },
      {
        message: "Автомобіль з таким VIN-кодом вже існує",
        path: ["vin"],
      },
    );

  const form = superForm(defaults(zod4(extendedSchema)), {
    SPA: true,
    validators: zod4Client(extendedSchema),
    validationMethod: "oninput",
    resetForm: false,
    id: formID,
    async onUpdate({ form: nativeForm, cancel }) {
      if (!nativeForm.valid) {
        addAnother = false;
        return;
      }

      const payload = {
        clientId: nativeForm.data.clientId,
        brandId: nativeForm.data.brandId,
        model: nativeForm.data.model,
        year: Number(nativeForm.data.year),
        licensePlate: normalizePlateText(nativeForm.data.licensePlate),
        plateType: nativeForm.data.plateType,
        plateCountry: nativeForm.data.plateCountry,
        vin: nativeForm.data.vin,
        engineCode: nativeForm.data.engineCode,
        engineDisplacement: nativeForm.data.engineDisplacement,
        fuelType: nativeForm.data.fuelType,
        driveType: nativeForm.data.driveType,
        notes: nativeForm.data.notes?.trim() || null,
      };

      submitError = null;
      const result =
        isEdit && carId
          ? await updateCarMutation
              .mutateAsync({ id: carId, payload })
              .catch(captureSubmitError)
          : await createCarMutation
              .mutateAsync(payload)
              .catch(captureSubmitError);

      if (!result) {
        addAnother = false;
        return;
      }

      cancel();

      if (!isEdit && addAnother) {
        addAnother = false;
        const keptOwner = nativeForm.data.clientId;
        form.reset();
        $formData.clientId = keptOwner;
        $formData.plateType = "ua";
        $formData.plateCountry = null;
        plateTypePreview = null;
        platePickerOpen = false;
        notesOpen = false;
        baseline = null;
        await tick();
        markBaseline();
        document.getElementById("car-brand-select")?.focus();
        toast.success("Автомобіль створено", {
          position: "bottom-left",
        });
        return;
      }

      baseline = null;
      form.reset();
      loadedKey = null;
      onCreated(result.id);
    },
  });

  const { form: formData, enhance, submitting } = form;

  const carModelsQuery = createCarModelsQuery(() => $formData.brandId);
  const modelSuggestions = $derived(carModelsQuery.data ?? []);
  const modelItems = $derived(modelSuggestions.map((s) => s.name));
  const modelSearchValues = $derived(
    new Map(
      modelSuggestions.map((s) => [s.name, [s.name, ...s.aliases].join(" ")]),
    ),
  );

  async function handleCreateBrand(name: string) {
    const brand = await createCarBrandMutation
      .mutateAsync(name)
      .catch(() => null);
    if (!brand) return;
    $formData.brandId = brand.id;
  }

  const vinWarning = $derived(
    ($formData.vin ?? "").length > 0 &&
      !VIN_RE.test(normalizeVin($formData.vin ?? "")),
  );

  const fuelOptions = [
    { value: "Gasoline", label: "Бензин", title: fuelTypeLabels.Gasoline },
    { value: "Diesel", label: "Дизель", title: fuelTypeLabels.Diesel },
    { value: "LpgCng", label: "Газ", title: fuelTypeLabels.LpgCng },
    { value: "Hybrid", label: "Гібрид", title: fuelTypeLabels.Hybrid },
    { value: "Electric", label: "Електро", title: fuelTypeLabels.Electric },
  ] satisfies { value: FuelType; label: string; title: string }[];

  const driveOptions = [
    {
      value: "FrontWheel",
      label: "Передній",
      title: driveTypeLabels.FrontWheel,
    },
    { value: "RearWheel", label: "Задній", title: driveTypeLabels.RearWheel },
    { value: "AllWheel", label: "Повний", title: driveTypeLabels.AllWheel },
    { value: "FourWheel", label: "4WD", title: driveTypeLabels.FourWheel },
  ] satisfies { value: DriveType; label: string; title: string }[];

  let platePickerOpen = $state(false);
  let plateTypePreview = $state<PlateType | null>(null);
  let plateInput = $state<HTMLInputElement | null>(null);

  let ownerEditing = $state(false);
  let ownerQuery = $state("");
  let notesOpen = $state(false);
  let baseline = $state<string | null>(null);
  let loadedKey = $state<string | null>(null);
  let addAnother = $state(false);
  let submitError = $state<string | null>(null);

  function captureSubmitError(error: unknown) {
    submitError =
      error instanceof Error ? error.message : "Не вдалося зберегти";
    return null;
  }

  const dirty = $derived(
    baseline !== null && JSON.stringify($formData) !== baseline,
  );

  function markBaseline() {
    baseline = JSON.stringify($formData);
  }

  const recentItemsQuery = createRecentItemsQuery();
  const recentClientIds = $derived(
    (recentItemsQuery.data ?? []).flatMap((item) =>
      item.kind === "client" ? [Number(item.id)] : [],
    ),
  );

  const ownerClient = $derived(
    clients.find((client) => client.id === $formData.clientId) ?? null,
  );
  const ownerLabel = $derived(
    ownerClient?.name ?? (ownerName.trim() !== "" ? ownerName : "Клієнт"),
  );

  function applyQueryPrefill(interpreted: ReturnType<typeof interpretQuery>) {
    if (interpreted.kind === "plate") {
      $formData.licensePlate = interpreted.value;
      return;
    }
    if (interpreted.kind !== "text") return;

    $formData.model = interpreted.value;
    const words = interpreted.words.map((word) => word.toLowerCase());
    const brand = brandList.find((item) => {
      const search = brandSearchValue(item).toLowerCase();
      return words.some((word) => search.split(/\s+/).includes(word));
    });
    if (brand) {
      $formData.brandId = brand.id;
      const brandWord = brand.name.toLowerCase();
      const rest = interpreted.words.filter(
        (word) => word.toLowerCase() !== brandWord,
      );
      if (rest.length > 0) $formData.model = rest.join(" ");
    }
  }

  $effect(() => {
    if (!active) {
      loadedKey = null;
      submitError = null;
      return;
    }

    const key = `${isEdit ? carId : "new"}:${ownerClientId ?? "-"}:${initialQuery}`;
    if (loadedKey === key) return;

    if (isEdit && !combinedQueries.isReady) return;
    if (!isEdit && combinedQueries.isLoading) return;

    loadedKey = key;
    ownerEditing = false;
    platePickerOpen = false;
    plateTypePreview = null;

    if (isEdit && carDto) {
      $formData.clientId = carDto.car.clientId;
      $formData.brandId = carDto.car.brandId;
      $formData.model = carDto.car.model;
      $formData.year = String(carDto.car.year);
      $formData.engineCode = carDto.car.engineCode;
      $formData.engineDisplacement = carDto.car.engineDisplacement;
      $formData.fuelType = carDto.car.fuelType;
      $formData.driveType = carDto.car.driveType;
      $formData.licensePlate = carDto.car.licensePlate;
      $formData.plateType = carDto.car.plateType ?? "ua";
      $formData.plateCountry = carDto.car.plateCountry ?? null;
      $formData.vin = carDto.car.vin;
      $formData.notes = carDto.car.notes;
      notesOpen = (carDto.car.notes ?? "").trim() !== "";
    } else {
      form.reset();
      notesOpen = false;
      if (ownerClientId != null) {
        $formData.clientId = ownerClientId;
      }
      ownerEditing = ownerClientId == null;
      const interpreted = interpretQuery(initialQuery);
      applyQueryPrefill(interpreted);
    }

    void tick().then(() => {
      markBaseline();
      document.getElementById("car-brand-select")?.focus();
      setTimeout(
        () => document.getElementById("car-brand-select")?.focus(),
        120,
      );
    });
  });

  function handleCancel() {
    form.reset();
    baseline = null;
    loadedKey = null;
    submitError = null;
    onCancel();
  }

  function submitAndAddAnother() {
    addAnother = true;
    (
      document.getElementById(formID) as HTMLFormElement | null
    )?.requestSubmit();
  }
</script>

{#snippet requiredMark()}
  <RequiredMark />
{/snippet}

{#snippet footerExtra()}
  {#if !isEdit}
    <Button
      type="button"
      variant="secondary"
      class={sheetSecondaryButton}
      disabled={$submitting}
      onclick={submitAndAddAnother}
    >
      Створити і додати ще
    </Button>
  {/if}
{/snippet}

<SheetFrame
  {formID}
  {active}
  {dirty}
  title={isEdit ? "Редагувати автомобіль" : "Створити автомобіль"}
  subtitle={isEdit
    ? "Змініть дані автомобіля"
    : "Внесіть дані нового автомобіля"}
  submitLabel={isEdit ? "Зберегти" : "Створити"}
  shortcutLabel={isEdit ? "зберегти" : "створити"}
  submitting={$submitting}
  error={submitError}
  enhanceAction={enhance}
  onRequestClose={handleCancel}
  {onBack}
  {backLabel}
  {stepLabel}
  {footerExtra}
  class="gap-0"
>
  <SheetSection title="Власник" class="pt-0">
    <Form.Field {form} name="clientId" class="space-y-1.5">
      <Form.Control>
        {#snippet children()}
          {#if ownerIsLocked}
            <OwnerCard name={ownerLabel} hint="Власник авто" />
          {:else if $formData.clientId >= 0 && !ownerEditing}
            <OwnerCard
              name={ownerLabel}
              hint="Власник авто"
              onEdit={() => {
                ownerQuery = "";
                ownerEditing = true;
              }}
            />
          {:else if ownerEditing}
            <ClientPicker
              {clients}
              {recentClientIds}
              bind:query={ownerQuery}
              autofocus
              {active}
              onSelect={(client) => {
                $formData.clientId = client.id;
                ownerQuery = "";
                ownerEditing = false;
              }}
              onClose={() => {
                ownerQuery = "";
                ownerEditing = false;
              }}
            />
          {:else}
            <button
              type="button"
              class="flex w-full items-center gap-3 rounded-[10px] border border-dashed border-border bg-card/60 px-3 py-2.5 text-left transition-colors hover:bg-muted/40"
              onclick={() => {
                ownerQuery = "";
                ownerEditing = true;
              }}
            >
              <span
                class="flex size-9 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground"
              >
                <UserIcon class="size-4" />
              </span>
              <span class="min-w-0 flex-1">
                <span class="block text-sm font-medium">Обрати власника</span>
                <span class="block text-xs text-muted-foreground">
                  Ім'я, телефон або email
                </span>
              </span>
              <span class="shrink-0 text-xs font-medium text-brand">
                Обрати
              </span>
            </button>
          {/if}
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>
  </SheetSection>

  <SheetSection title="Автомобіль">
    <div class="flex flex-col gap-4">
      <div class="grid grid-cols-1 gap-x-3 gap-y-4 sm:grid-cols-2">
        <Form.Field {form} name="brandId" class="space-y-1.5">
          <Form.Control>
            {#snippet children({ props: formProps })}
              <Label class={sheetLabel}>
                Марка{@render requiredMark()}
              </Label>
              <ComboSelect
                items={brandList}
                bind:value={$formData.brandId}
                getKey={(brand) => brand.id}
                getLabel={(brand) => brand.name}
                getSearchValue={brandSearchValue}
                placeholder="Оберіть марку..."
                searchPlaceholder="Пошук марки..."
                emptyText="Марку не знайдено."
                triggerClass={cn(sheetInput, "justify-between")}
                contentClass="w-(--bits-floating-anchor-width) min-w-56 p-0"
                onCreate={handleCreateBrand}
                creating={createCarBrandMutation.isPending}
                formProps={{
                  ...formProps,
                  id: "car-brand-select",
                  "data-sheet-autofocus": "",
                }}
              />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>

        <Form.Field {form} name="model" class="space-y-1.5">
          <Form.Control>
            {#snippet children({ props })}
              <Label class={sheetLabel}>
                Модель{@render requiredMark()}
              </Label>
              <SuggestInput
                {...props}
                items={modelItems}
                getSearchValue={(name) => modelSearchValues.get(name) ?? name}
                bind:value={$formData.model}
                class={sheetInput}
                placeholder="напр. Camry"
              />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>
      </div>

      <div class="grid grid-cols-1 gap-x-3 gap-y-4 sm:grid-cols-2">
        <Form.Field {form} name="year" class="space-y-1.5">
          <Form.Control>
            {#snippet children({ props })}
              <Label class={sheetLabel}>
                Рік випуску{@render requiredMark()}
              </Label>
              <Input
                {...props}
                class={sheetInput}
                bind:value={$formData.year}
                inputmode="numeric"
                placeholder="напр. 2019"
                maxlength={4}
              />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>

        <Form.Field {form} name="licensePlate" class="space-y-1.5">
          <Form.Control>
            {#snippet children({ props })}
              <Label class={sheetLabel}>Номерний знак</Label>
              <div
                class={cn(
                  "flex items-stretch rounded-[8px] border border-border bg-muted/40 transition-[color,box-shadow]",
                  sheetControlHeight,
                  platePickerOpen
                    ? "border-brand ring-3 ring-brand/20"
                    : "focus-within:border-brand focus-within:ring-3 focus-within:ring-brand/20",
                )}
              >
                <PlateTypePicker
                  plateType={$formData.plateType ?? "ua"}
                  plateCountry={$formData.plateCountry ?? null}
                  bind:previewType={plateTypePreview}
                  bind:open={platePickerOpen}
                  onChange={(type, country) => {
                    $formData.plateType = type;
                    $formData.plateCountry = country;
                    void tick().then(() => {
                      plateInput?.focus();
                      setTimeout(() => plateInput?.focus(), 40);
                    });
                  }}
                />

                <Input
                  bind:ref={plateInput}
                  {...props}
                  value={$formData.licensePlate ?? ""}
                  oninput={(event) => {
                    $formData.licensePlate = event.currentTarget.value;
                  }}
                  class="h-full min-w-0 flex-1 rounded-r-[7px] rounded-l-none border-0 bg-transparent px-2.5 font-medium uppercase shadow-none focus-visible:ring-0"
                  placeholder={plateTypeMeta(
                    plateTypePreview ?? $formData.plateType,
                  ).placeholder}
                />
              </div>
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>
      </div>

      <Form.Field {form} name="vin" class="space-y-1.5">
        <Form.Control>
          {#snippet children({ props })}
            <Label class={sheetLabel}>VIN-код</Label>
            <div class="relative">
              <Input
                {...props}
                value={$formData.vin ?? ""}
                oninput={(event) => {
                  $formData.vin = normalizeVin(event.currentTarget.value);
                }}
                class={cn(sheetInput, "pr-14 font-mono")}
                placeholder="17 символів"
                maxlength={17}
                spellcheck="false"
              />
              <span
                class="pointer-events-none absolute top-1/2 right-3 -translate-y-1/2 text-[11px] tabular-nums text-muted-foreground/70"
              >
                {($formData.vin ?? "").length}/17
              </span>
            </div>
            {#if vinWarning}
              <p
                class="text-xs leading-relaxed text-amber-600 dark:text-amber-500"
              >
                VIN зазвичай має 17 символів (без I, O та Q). Перевірте номер,
                якщо це можливо.
              </p>
            {/if}
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>
    </div>
  </SheetSection>

  <SheetSection title="Двигун і трансмісія">
    <div class="flex flex-col gap-4">
      <div class="grid grid-cols-1 gap-x-3 gap-y-4 sm:grid-cols-2">
        <Form.Field {form} name="engineCode" class="space-y-1.5">
          <Form.Control>
            {#snippet children({ props })}
              <Label class={sheetLabel}>Код двигуна</Label>
              <Input
                {...props}
                value={$formData.engineCode ?? ""}
                oninput={(event) => {
                  $formData.engineCode = event.currentTarget.value;
                }}
                class={cn(sheetInput, "font-mono")}
                placeholder="напр. CAXA"
              />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>

        <Form.Field {form} name="engineDisplacement" class="space-y-1.5">
          <Form.Control>
            {#snippet children({ props })}
              <Label class={sheetLabel}>Об'єм двигуна</Label>
              <div class="relative">
                <Input
                  {...props}
                  bind:value={$formData.engineDisplacement}
                  class={cn(sheetInput, "pr-8")}
                  type="number"
                  step="0.1"
                  min="0"
                  placeholder="напр. 2.0"
                />
                <span
                  class="pointer-events-none absolute top-1/2 right-3 -translate-y-1/2 text-xs whitespace-nowrap text-muted-foreground"
                >
                  л.
                </span>
              </div>
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>
      </div>

      <Form.Field {form} name="fuelType" class="space-y-1.5">
        <Form.Control>
          {#snippet children()}
            <Label class={sheetLabel}>Тип палива</Label>
            <SegmentedControl
              options={fuelOptions}
              value={$formData.fuelType}
              ariaLabel="Тип палива"
              onValueChange={(value) => {
                $formData.fuelType = value as FuelType | null;
              }}
            />
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>

      <Form.Field {form} name="driveType" class="space-y-1.5">
        <Form.Control>
          {#snippet children()}
            <Label class={sheetLabel}>Тип приводу</Label>
            <SegmentedControl
              options={driveOptions}
              value={$formData.driveType}
              ariaLabel="Тип приводу"
              onValueChange={(value) => {
                $formData.driveType = value as DriveType | null;
              }}
            />
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>
    </div>
  </SheetSection>

  <SheetSection title="Додатково">
    <NotesDisclosure bind:expanded={notesOpen}>
      <Form.Field {form} name="notes" class="space-y-1.5">
        <Form.Control>
          {#snippet children({ props })}
            <Label class={sheetLabel}>Нотатки</Label>
            <Textarea
              {...props}
              class={sheetTextarea}
              value={$formData.notes ?? ""}
              oninput={(event) => {
                $formData.notes = event.currentTarget.value;
              }}
              placeholder="Додаткова інформація про автомобіль..."
              rows={3}
            />
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>
    </NotesDisclosure>
  </SheetSection>
</SheetFrame>

<script lang="ts">
  import { createQueries, useQueryClient } from "@tanstack/svelte-query";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select";
  import { Textarea } from "$lib/components/ui/textarea";

  import ComboSelect from "$lib/components/combo-select.svelte";
  import MutationSheetShell from "$lib/components/mutation-sheet-shell.svelte";

  import type { DriveType, FuelType } from "$lib/bindings";
  import { cn } from "$lib/utils";

  import ClientBadge from "../clients/client-badge.svelte";
  import { clientsQueryOptions } from "../clients/queries";
  import { driveTypeLabels, fuelTypeLabels } from "./enum-labels";
  import {
    carBrandsQueryOptions,
    carsListQueryOptions,
    createCreateCarMutation,
    createUpdateCarMutation,
  } from "./queries";

  let {
    carId = null,
    initClientId = null,
    open = $bindable(false),
  }: {
    carId?: number | null;
    initClientId?: number | null;
    open: boolean;
  } = $props();

  const formID = "car-mutation-form";
  const isEdit = $derived(carId != null);

  const queryClient = useQueryClient();

  const createCarMutation = createCreateCarMutation(queryClient);
  const updateCarMutation = createUpdateCarMutation(queryClient);

  const combinedQueries = createQueries(() => ({
    queries: [
      carsListQueryOptions(),
      clientsQueryOptions(),
      carBrandsQueryOptions(),
    ],
    combine: (results) => {
      return {
        cars: results[0],
        clients: results[1],
        carBrands: results[2],

        isReady: results.every((r) => r.isSuccess),
        isLoading: results.some((r) => r.isPending),
        isError: results.some((r) => r.isError),
        firstError: results.find((r) => r.isError)?.error,

        refetchAll: () => {
          results.forEach((r) => r.refetch());
        },
      };
    },
  }));

  let carsList = $derived(combinedQueries.cars.data ?? []);
  let carDto = $derived(carsList.find((c) => c.car.id === carId) ?? null);

  // Standard 2004+ format: 2 region letters, 4 digits, 2 series letters.
  // Only Cyrillic letters that resemble Latin ones are used on real UA plates.
  const STANDARD_PLATE_RE =
    /^[ABEIKMHOPCTXАВЕІКМНОРСТХ]{2}\d{4}[ABEIKMHOPCTXАВЕІКМНОРСТХ]{2}$/;

  function normalizePlate(value: string) {
    return value.toUpperCase().replace(/[\s-]/g, "");
  }

  function normalizeVin(value: string) {
    return value.toUpperCase().replace(/[\s-]/g, "");
  }

  const fuelTypeOptions = Object.entries(fuelTypeLabels).map(
    ([value, label]) => ({ value: value as FuelType, label }),
  );

  const driveTypeOptions = Object.entries(driveTypeLabels).map(
    ([value, label]) => ({ value: value as DriveType, label }),
  );

  const carSchema = z.object({
    clientId: z
      .number({ error: "Обов'язкове поле" })
      .min(0, { error: "Обов'язкове поле" })
      .default(-1),

    brandId: z
      .number({ error: "Обов'язкове поле" })
      .min(0, { error: "Обов'язкове поле" })
      .default(-1),

    model: z.string().min(1, { error: "Обов'язкове поле" }),

    year: z
      .number({ error: "Обов'язкове поле" })
      .min(1885, { error: "Неправильний рік" })
      .max(new Date().getFullYear() + 1, {
        error: "Рік не може бути в далекому майбутньому",
      })
      .default(0),
    licensePlate: z
      .string()
      .nullable()
      .default(null)
      .transform((val) => (val ? normalizePlate(val) : null))
      .transform((val) => (val === "" ? null : val))
      .pipe(
        z
          .string()
          .regex(/^[A-ZА-Я0-9]*$/, {
            error: "Номерний знак містить неприпустимі символи",
          })
          .nullable(), // Allow null to pass through the pipe
      ),

    vin: z
      .string()
      .nullable()
      .default(null)
      .transform((val) => (val ? normalizeVin(val) : null))
      .transform((val) => (val === "" ? null : val))
      .pipe(
        z
          .string()
          .regex(/^[A-HJ-NPR-Z0-9]*$/, {
            error: "VIN-код містить неприпустимі символи",
            abort: true,
          })
          .length(17, { error: "Довжина VIN-коду має становити 17 символів" })
          .nullable(),
      ),

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

  //TODO: add the option to navigate directly to already existsing car if it was found
  const extendedSchema = carSchema
    .refine(
      (data) => {
        if (!data.licensePlate) return true;

        const duplicateCar = carsList.find(
          (dto) => dto.car.licensePlate === data.licensePlate,
        );
        if (!duplicateCar) return true;
        if (isEdit && duplicateCar?.car?.id === carDto?.car?.id) return true;
        return false;
      },
      {
        message: "Автомобіль з таким реєстраційним номером вже існує",
        path: ["licensePlate"],
      },
    )
    .refine(
      (data) => {
        if (!data.vin) return true;

        const duplicateCar = carsList.find((dto) => dto?.car?.vin === data.vin);
        if (!duplicateCar) return true;
        if (isEdit && duplicateCar.car?.id === carDto?.car?.id) return true;
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
    async onUpdate({ form: nativeForm }) {
      if (!nativeForm.valid) return;

      const payload = {
        clientId: nativeForm.data.clientId,
        brandId: nativeForm.data.brandId,
        model: nativeForm.data.model,
        year: nativeForm.data.year,
        licensePlate: nativeForm.data.licensePlate,
        vin: nativeForm.data.vin,
        engineCode: nativeForm.data.engineCode,
        engineDisplacement: nativeForm.data.engineDisplacement,
        fuelType: nativeForm.data.fuelType,
        driveType: nativeForm.data.driveType,
        notes: nativeForm.data.notes?.trim() || null,
      };

      const result =
        isEdit && carId
          ? await updateCarMutation
              .mutateAsync({ id: carId, payload })
              .catch(() => null)
          : await createCarMutation.mutateAsync(payload).catch(() => null);

      if (!result) return;
      open = false;
      form.reset();
    },
  });

  const { form: formData, enhance, submitting } = form;

  const licensePlateLooksNonStandard = $derived(
    $formData.licensePlate != null &&
      $formData.licensePlate.length > 0 &&
      !STANDARD_PLATE_RE.test($formData.licensePlate),
  );

  $effect(() => {
    if (open) {
      if (initClientId) {
        $formData.clientId = initClientId;
        return;
      }
      if (carDto) {
        $formData.clientId = carDto.car.clientId;
        $formData.brandId = carDto.car.brandId;
        $formData.model = carDto.car.model;
        $formData.year = carDto.car.year;
        $formData.engineCode = carDto.car.engineCode;
        $formData.engineDisplacement = carDto.car.engineDisplacement;
        $formData.fuelType = carDto.car.fuelType;
        $formData.driveType = carDto.car.driveType;
        $formData.licensePlate = carDto.car.licensePlate;
        $formData.vin = carDto.car.vin;
        $formData.notes = carDto.car.notes;
      } else {
        form.reset();
      }
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

<MutationSheetShell
  bind:open
  {formID}
  title={isEdit ? "Редагувати автомобіль" : "Створити автомобіль"}
  description={isEdit
    ? "Змініть дані автомобіля"
    : "Внесіть дані нового автомобіля"}
  submitLabel={isEdit ? "Зберегти зміни" : "Створити"}
  submitting={$submitting}
  enhanceAction={enhance}
  onCancel={() => {
    form.reset();
    open = false;
  }}
>
  <Form.Field {form} name="clientId">
    <Form.Control>
      {#snippet children({ props: formProps })}
        <Label>Власник {@render requiredHint()}</Label>
        <ComboSelect
          items={combinedQueries.clients.data ?? []}
          bind:value={$formData.clientId}
          disabled={initClientId != null}
          getKey={(c) => c.id}
          getLabel={(c) => c.name}
          getSearchValue={(c) => c.name}
          placeholder="Оберіть власника..."
          searchPlaceholder="Пошук клієнтів..."
          emptyText="Клієнта не знайдено."
          triggerClass={cn($formData.clientId != -1 && "pl-2")}
          {formProps}
        >
          {#snippet trigger(selected)}
            {#if selected}
              <ClientBadge name={selected.name} iconSize="size-5" />
            {:else}
              Оберіть власника...
            {/if}
          {/snippet}
          {#snippet item(client)}
            <ClientBadge name={client.name} />
          {/snippet}
        </ComboSelect>
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="brandId">
    <Form.Control>
      {#snippet children({ props: formProps })}
        <Label>Марка {@render requiredHint()}</Label>
        <ComboSelect
          items={combinedQueries.carBrands.data ?? []}
          bind:value={$formData.brandId}
          getKey={(b) => b.id}
          getLabel={(b) => b.name}
          placeholder="Оберіть марку авто..."
          searchPlaceholder="Пошук марки..."
          emptyText="Марку не знайдено."
          contentClass="w-48 p-0"
          {formProps}
        />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="model">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Модель {@render requiredHint()}</Label>
        <Input {...props} bind:value={$formData.model} />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="year">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Рік випуску {@render requiredHint()}</Label>
        <Input type="number" {...props} bind:value={$formData.year} />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="engineCode">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Код двигуна</Label>
        <Input
          {...props}
          value={$formData.engineCode ?? ""}
          oninput={(e) => {
            $formData.engineCode = e.currentTarget.value;
          }}
          placeholder="напр. CAXA"
        />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="engineDisplacement">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Об'єм двигуна</Label>
        <div class="flex gap-2 items-baseline" {...props}>
          <Input
            bind:value={$formData.engineDisplacement}
            type="number"
            step="0.1"
            min="0"
            {...props}
          />
          <p>л.</p>
        </div>
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="fuelType">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Тип палива</Label>
        <Select.Root
          type="single"
          bind:value={
            () => $formData.fuelType ?? "",
            (v) => ($formData.fuelType = v === "" ? null : (v as FuelType))
          }
        >
          <Select.Trigger class="select-none" {...props}>
            {$formData.fuelType
              ? fuelTypeLabels[$formData.fuelType]
              : "Не вказано"}
          </Select.Trigger>
          <Select.Content>
            <Select.Item value="" label="Не вказано" />
            {#each fuelTypeOptions as option (option.value)}
              <Select.Item value={option.value} label={option.label} />
            {/each}
          </Select.Content>
        </Select.Root>
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="driveType">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Тип приводу</Label>
        <Select.Root
          type="single"
          bind:value={
            () => $formData.driveType ?? "",
            (v) => ($formData.driveType = v === "" ? null : (v as DriveType))
          }
        >
          <Select.Trigger class="select-none" {...props}>
            {$formData.driveType
              ? driveTypeLabels[$formData.driveType]
              : "Не вказано"}
          </Select.Trigger>
          <Select.Content>
            <Select.Item value="" label="Не вказано" />
            {#each driveTypeOptions as option (option.value)}
              <Select.Item value={option.value} label={option.label} />
            {/each}
          </Select.Content>
        </Select.Root>
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="licensePlate">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Номерний знак</Label>
        <Input
          {...props}
          value={$formData.licensePlate ?? ""}
          oninput={(e) => {
            $formData.licensePlate = normalizePlate(e.currentTarget.value);
          }}
          class="uppercase"
        />
        {#if licensePlateLooksNonStandard}
          <p class="text-xs text-muted-foreground">
            Не схоже на стандартний номер — це нормально для старих, тимчасових
            чи спецтранспортних номерів.
          </p>
        {/if}
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="vin">
    <Form.Control>
      {#snippet children({ props })}
        <Label>VIN-код</Label>
        <Input
          {...props}
          value={$formData.vin ?? ""}
          oninput={(e) => {
            $formData.vin = normalizeVin(e.currentTarget.value);
          }}
          class="uppercase"
        />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="notes">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Нотатки</Label>
        <Textarea
          {...props}
          value={$formData.notes ?? ""}
          oninput={(e) => {
            $formData.notes = e.currentTarget.value;
          }}
          placeholder="Додаткова інформація про автомобіль..."
          rows={3}
        />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>
</MutationSheetShell>

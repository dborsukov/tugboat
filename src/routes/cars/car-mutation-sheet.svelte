<script lang="ts">
  import { createQueries, useQueryClient } from "@tanstack/svelte-query";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";

  import ComboSelect from "$lib/components/combo-select.svelte";
  import MutationSheetShell from "$lib/components/mutation-sheet-shell.svelte";

  import { cn } from "$lib/utils";

  import ClientBadge from "../clients/client-badge.svelte";
  import { clientsQueryOptions } from "../clients/queries";
  import {
    carBrandsQueryOptions,
    carsListQueryOptions,
    createCreateCarMutation,
    createUpdateCarMutation,
  } from "./queries";

  let {
    carId = null,
    open = $bindable(false),
  }: { carId?: number | null; open: boolean } = $props();

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

  const carSchema = z.object({
    clientId: z
      .number({ error: "Вкажіть власника" })
      .min(0, { error: "Вкажіть власника" })
      .default(-1),

    brandId: z
      .number({ error: "Вкажіть марку" })
      .min(0, { error: "Вкажіть марку" })
      .default(-1),

    model: z.string().min(1, { error: "Вкажіть модель" }),

    year: z
      .number({ error: "Вкажіть рік" })
      .min(1885, { error: "Неправильний рік" })
      .max(new Date().getFullYear() + 1, {
        error: "Рік не може бути в далекому майбутньому",
      })
      .default(() => new Date().getFullYear()),

    licensePlate: z
      .string()
      .trim()
      .transform(normalizePlate)
      .pipe(
        z
          .string()
          .min(1, { error: "Занадто короткий номерний знак" })
          .regex(/^[A-ZА-Я0-9]*$/, {
            error: "Номерний знак містить неприпустимі символи",
          }),
      ),

    vin: z
      .string()
      .trim()
      .transform(normalizeVin)
      .pipe(
        z
          .string()
          .regex(/^[A-HJ-NPR-Z0-9]*$/, {
            error: "VIN-код містить неприпустимі символи",
            abort: true,
          })
          .length(17, { error: "Довжина VIN-коду має становити 17 символів" }),
      ),
  });

  //TODO: add the option to navigate directly to already existsing car if it was found
  const extendedSchema = carSchema
    .refine(
      (data) => {
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
    $formData.licensePlate.length > 0 &&
      !STANDARD_PLATE_RE.test($formData.licensePlate),
  );

  $effect(() => {
    if (open) {
      if (carDto) {
        $formData.clientId = carDto.car.clientId;
        $formData.brandId = carDto.car.brandId;
        $formData.model = carDto.car.model;
        $formData.year = carDto.car.year;
        $formData.licensePlate = carDto.car.licensePlate ?? "";
        $formData.vin = carDto.car.vin ?? "";
      } else {
        form.reset();
      }
    }
  });
</script>

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
        <Label>Власник</Label>
        <ComboSelect
          items={combinedQueries.clients.data ?? []}
          bind:value={$formData.clientId}
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
        <Label>Марка</Label>
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
        <Label>Модель</Label>
        <Input {...props} bind:value={$formData.model} />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="year">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Рік випуску</Label>
        <Input type="number" {...props} bind:value={$formData.year} />
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
          value={$formData.licensePlate}
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
          value={$formData.vin}
          oninput={(e) => {
            $formData.vin = normalizeVin(e.currentTarget.value);
          }}
          class="uppercase"
        />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>
</MutationSheetShell>

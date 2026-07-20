<script lang="ts">
  import { tick } from "svelte";
  import { toast } from "svelte-sonner";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import CheckIcon from "@lucide/svelte/icons/check";
  import ChevronsUpDownIcon from "@lucide/svelte/icons/chevrons-up-down";

  import { commands } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import { cn } from "$lib/utils";

  import { Button } from "$lib/components/ui/button";
  import * as Command from "$lib/components/ui/command";
  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Popover from "$lib/components/ui/popover";
  import * as Sheet from "$lib/components/ui/sheet";
  import { Spinner } from "$lib/components/ui/spinner";

  import ClientBadge from "../clients/client-badge.svelte";

  interface Props {
    open: boolean;
    car?: any | null;
  }

  let { open = $bindable(false), car = null }: Props = $props();

  const isEdit = $derived(!!car);
  const titleText = $derived(
    isEdit ? "Редагувати автомобіль" : "Створити автомобіль",
  );
  const descText = $derived(
    isEdit
      ? "Змініть дані автомобіля тут."
      : "Внесіть дані нового автомобіля тут.",
  );
  const formID = "car-mutation-form";

  let clientComboOpen = $state(false);
  let clientComboTriggerRef = $state<HTMLButtonElement>(null!);
  let brandComboOpen = $state(false);
  let brandComboTriggerRef = $state<HTMLButtonElement>(null!);

  const brandComboSelectedValue = $derived(
    appStore.car_brands.find((b) => b.id === $formData.brand_id)?.name,
  );

  // Standard 2004+ format: 2 region letters, 4 digits, 2 series letters.
  // Only Cyrillic letters that resemble Latin ones are used on real UA plates.
  const STANDARD_PLATE_RE =
    /^[ABEIKMHOPCTXАВЕІКМНОРСТХ]{2}\d{4}[ABEIKMHOPCTXАВЕІКМНОРСТХ]{2}$/;

  function normalizePlate(value: string) {
    return value.toUpperCase().replace(/[\s-]/g, "");
  }

  const carSchema = z.object({
    client_id: z
      .number({ error: "Вкажіть власника" })
      .min(0, { error: "Вкажіть власника" })
      .default(-1),

    brand_id: z
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

    registration: z
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
      .toUpperCase()
      .regex(/^[A-HJ-NPR-Z0-9]*$/, {
        error: "VIN-код містить неприпустимі символи",
        abort: true,
      })
      .length(17, { error: "Довжина VIN-коду має становити 17 символів" }),
  });

  //TODO: add the option to navigate directly to already existsing car if it was found
  const extendedSchema = carSchema
    .refine(
      (data) => {
        const carsList = appStore.cars || [];
        const duplicateCar = carsList.find(
          (c) => c.registration === data.registration,
        );
        if (!duplicateCar) return true;
        // Avoid record matching with itself (when editing)
        if (isEdit && duplicateCar.id === car?.id) return true;
        return false;
      },
      {
        message: "Автомобіль з таким реєстраційним номером вже існує",
        path: ["registration"],
      },
    )
    .refine(
      (data) => {
        const carsList = appStore.cars || [];
        const duplicateCar = carsList.find((c) => c.vin === data.vin);
        if (!duplicateCar) return true;
        // Avoid record matching with itself (when editing)
        if (isEdit && duplicateCar.id === car?.id) return true;
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
        client_id: nativeForm.data.client_id,
        brand_id: nativeForm.data.brand_id,
        model: nativeForm.data.model,
        year: nativeForm.data.year,
        registration: nativeForm.data.registration,
        vin: nativeForm.data.vin,
      };

      if (isEdit) {
        let res = await commands.updateCar(car.id, payload);
        if (res.status === "ok") {
          appStore.updateCar(res.data);
        } else {
          return toast.error("Виникла помилка", {
            description: res.error.message,
            position: "bottom-left",
          });
        }
      } else {
        let res = await commands.createCar(payload);
        if (res.status === "ok") {
          appStore.createCar(res.data);
        } else {
          return toast.error("Виникла помилка", {
            description: res.error.message,
            position: "bottom-left",
          });
        }
      }
      open = false;
      form.reset();
    },
  });

  const { form: formData, enhance, submitting } = form;

  const registrationLooksNonStandard = $derived(
    $formData.registration.length > 0 &&
      !STANDARD_PLATE_RE.test($formData.registration),
  );

  $effect(() => {
    if (open) {
      if (car) {
        $formData.client_id = car.client_id;
        $formData.brand_id = car.brand_id;
        $formData.model = car.model;
        $formData.year = car.year;
        $formData.registration = car.registration ?? "";
        $formData.vin = car.vin ?? "";
      } else {
        form.reset();
      }
    }
  });

  function closeAndFocusClient() {
    clientComboOpen = false;
    tick().then(() => clientComboTriggerRef?.focus());
  }

  function closeAndFocusBrand() {
    brandComboOpen = false;
    tick().then(() => brandComboTriggerRef?.focus());
  }

  function handleCancel() {
    form.reset();
    open = false;
  }
</script>

<Sheet.Root bind:open>
  <Sheet.Content
    side="right"
    showCloseButton={false}
    onEscapeKeydown={(e) => e.preventDefault()}
    onInteractOutside={(e) => e.preventDefault()}
  >
    <Sheet.Header>
      <Sheet.Title>{titleText}</Sheet.Title>
      <Sheet.Description>{descText}</Sheet.Description>
    </Sheet.Header>

    <form
      use:enhance
      id={formID}
      class="flex flex-col p-6 gap-4 overflow-y-auto h-full"
    >
      <Form.Field {form} name="client_id">
        <Form.Control>
          {#snippet children({ props: formProps })}
            <Label>Власник</Label>
            <Popover.Root bind:open={clientComboOpen}>
              <Popover.Trigger bind:ref={clientComboTriggerRef}>
                {#snippet child({ props: triggerProps })}
                  <Button
                    {...formProps}
                    {...triggerProps}
                    variant="outline"
                    class={cn(
                      "w-full justify-between",
                      $formData.client_id != -1 && "pl-2",
                    )}
                    role="combobox"
                    aria-expanded={clientComboOpen}
                  >
                    {#if $formData.client_id != -1}
                      <ClientBadge
                        clientId={$formData.client_id}
                        iconSize="size-5"
                      />
                    {:else}
                      Оберіть власника...
                    {/if}
                    <ChevronsUpDownIcon class="opacity-50" />
                  </Button>
                {/snippet}
              </Popover.Trigger>
              <Popover.Content class="w-full p-0" align="start">
                <Command.Root>
                  <Command.Input placeholder="Пошук клієнтів..." />
                  <Command.List>
                    <Command.Empty>Клієнта не знайдено.</Command.Empty>
                    <Command.Group>
                      {#each appStore.clients as client (client.id)}
                        <Command.Item
                          value={`${client.name}-${client.id}`}
                          onSelect={() => {
                            $formData.client_id = client.id;
                            closeAndFocusClient();
                          }}
                        >
                          <CheckIcon
                            class={cn(
                              $formData.client_id !== client.id &&
                                "text-transparent",
                            )}
                          />
                          <ClientBadge {client} />
                        </Command.Item>
                      {/each}
                    </Command.Group>
                  </Command.List>
                </Command.Root>
              </Popover.Content>
            </Popover.Root>
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>

      <Form.Field {form} name="brand_id">
        <Form.Control>
          {#snippet children({ props: formProps })}
            <Label>Марка</Label>
            <Popover.Root bind:open={brandComboOpen}>
              <Popover.Trigger bind:ref={brandComboTriggerRef}>
                {#snippet child({ props: triggerProps })}
                  <Button
                    {...formProps}
                    {...triggerProps}
                    variant="outline"
                    class="w-full justify-between"
                    role="combobox"
                    aria-expanded={brandComboOpen}
                  >
                    {brandComboSelectedValue || "Оберіть марку авто..."}
                    <ChevronsUpDownIcon class="opacity-50" />
                  </Button>
                {/snippet}
              </Popover.Trigger>
              <Popover.Content class="w-50 p-0" align="start">
                <Command.Root>
                  <Command.Input placeholder="Пошук марки..." />
                  <Command.List>
                    <Command.Empty>Марку не знайдено.</Command.Empty>
                    <Command.Group>
                      {#each appStore.car_brands as brand (brand.id)}
                        <Command.Item
                          value={brand.name}
                          onSelect={() => {
                            $formData.brand_id = brand.id;
                            closeAndFocusBrand();
                          }}
                        >
                          <CheckIcon
                            class={cn(
                              $formData.brand_id !== brand.id &&
                                "text-transparent",
                            )}
                          />
                          {brand.name}
                        </Command.Item>
                      {/each}
                    </Command.Group>
                  </Command.List>
                </Command.Root>
              </Popover.Content>
            </Popover.Root>
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

      <Form.Field {form} name="registration">
        <Form.Control>
          {#snippet children({ props })}
            <Label>Реєстраційний номер</Label>
            <Input
              {...props}
              value={$formData.registration}
              oninput={(e) => {
                $formData.registration = normalizePlate(e.currentTarget.value);
              }}
              class="uppercase"
            />
            {#if registrationLooksNonStandard}
              <p class="text-xs text-muted-foreground">
                Не схоже на стандартний номер — це нормально для старих,
                тимчасових чи спецтранспортних номерів.
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
                $formData.vin = e.currentTarget.value.toUpperCase();
              }}
              class="uppercase"
            />
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>
    </form>
    <Sheet.Footer>
      <Button type="submit" disabled={$submitting} form={formID}>
        {#if $submitting}<Spinner />{/if}
        {isEdit ? "Зберегти зміни" : "Створити"}
      </Button>
      <Button variant="outline" disabled={$submitting} onclick={handleCancel}>
        Відмінити та закрити
      </Button>
    </Sheet.Footer>
  </Sheet.Content>
</Sheet.Root>

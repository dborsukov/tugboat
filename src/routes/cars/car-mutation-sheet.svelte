<script lang="ts">
  import { tick } from "svelte";
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

  const clientComboSelectedValue = $derived(
    appStore.clients.find((b) => b.id === $formData.client_id)?.name,
  );
  const brandComboSelectedValue = $derived(
    appStore.car_brands.find((b) => b.id === $formData.brand_id)?.name,
  );

  const vehicleSchema = z.object({
    client_id: z
      .number({ error: "Вкажіть власника" })
      .min(1, { error: "Вкажіть власника" })
      .default(-1),

    brand_id: z
      .number({ error: "Вкажіть марку" })
      .min(1, { error: "Вкажіть марку" })
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
      .toUpperCase()
      .min(1, { error: "Вкажіть реєстраційний номер" }),

    vin: z
      .string()
      .trim()
      .toUpperCase()
      .regex(/^[A-HJ-NPR-Z0-9]{17}$/, {
        error: "VIN-код має складатись з 17 правильних символів",
      }),
  });

  const form = superForm(defaults(zod4(vehicleSchema)), {
    SPA: true,
    validators: zod4Client(vehicleSchema),
    validationMethod: "oninput",
    resetForm: false,
    id: formID,
    async onUpdate({ form: nativeForm }) {
      if (!nativeForm.valid) return;

      const payload = {
        client_id: $formData.client_id,
        brand_id: $formData.brand_id,
        model: $formData.model,
        year: $formData.year,
        registration: $formData.registration,
        vin: $formData.vin,
      };

      if (isEdit) {
        let res = await commands.updateCar(car.id, payload);
        if (res.status === "ok") {
          appStore.updateCar(res.data);
        } else {
          // TODO: show error to user
          alert(res.error);
        }
      } else {
        let res = await commands.createCar(payload);
        if (res.status === "ok") {
          appStore.createCar(res.data);
        } else {
          // TODO: show error to user
          alert(res.error);
        }
      }
      open = false;
      form.reset();
    },
  });

  const { form: formData, errors, enhance, submitting } = form;

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
                    class="w-full justify-between"
                    role="combobox"
                    aria-expanded={clientComboOpen}
                  >
                    {clientComboSelectedValue || "Оберіть власника..."}
                    <ChevronsUpDownIcon class="opacity-50" />
                  </Button>
                {/snippet}
              </Popover.Trigger>
              <Popover.Content class="w-50 p-0" align="start">
                <Command.Root>
                  <Command.Input placeholder="Пошук клієнтів..." />
                  <Command.List>
                    <Command.Empty>Клієнта не знайдено.</Command.Empty>
                    <Command.Group>
                      {#each appStore.clients as client (client.id)}
                        <Command.Item
                          value={client.name}
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
                          {client.name}
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
              bind:value={$formData.registration}
              class="uppercase font-mono font-semibold"
            />
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
              bind:value={$formData.vin}
              class="uppercase font-mono font-semibold"
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

<script lang="ts">
  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import SquarePenIcon from "@lucide/svelte/icons/square-pen";
  import TrashIcon from "@lucide/svelte/icons/trash-2";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import { goto } from "$app/navigation";

  import { commands, type Client } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js";
  import ButtonGroup from "$lib/components/ui/button-group/button-group.svelte";
  import Button, {
    buttonVariants,
  } from "$lib/components/ui/button/button.svelte";
  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Sheet from "$lib/components/ui/sheet";
  import Spinner from "$lib/components/ui/spinner/spinner.svelte";

  import PageLayout from "$lib/components/page-layout.svelte";

  let { client }: { client: Client } = $props();

  let isDeleting = $state(false);
  let isSheetOpen = $state(false);

  export const dateFormatter = new Intl.DateTimeFormat("uk-UA", {
    day: "numeric",
    month: "long",
    year: "numeric",
  });

  export const userSchema = z.object({
    name: z.string().nonempty("Ім'я не може бути пустим"),
    phone: z.string().nonempty("Мобільний номер не може бути пустим"),
  });

  // svelte-ignore state_referenced_locally
  const defaultFormData = {
    name: client.name,
    phone: client.phone ?? "",
  };

  const form = superForm(defaults(defaultFormData, zod4(userSchema)), {
    SPA: true,
    validators: zod4Client(userSchema),
    validationMethod: "oninput",
    resetForm: false,
    async onUpdate({ form }) {
      if (!form.valid) return;
      try {
        let res = await commands.updateClient(client.id, {
          name: form.data.name,
          phone: form.data.phone,
        });
        if (res.status === "ok") {
          let updated_client = res.data;
          const index = appStore.clients.findIndex(
            (c) => c.id === updated_client.id,
          );
          if (index !== -1) {
            appStore.clients[index] = updated_client;
          }
          isSheetOpen = false;
        } else {
          // TODO: show error to user
          alert(res.error);
        }
      } catch (err) {
        // TODO: show error to user
        alert(`Critical IPC error: ${err}`);
      }
    },
  });
  const { form: formData, errors, enhance, submitting } = form;

  function handleCancel() {
    form.reset();
    isSheetOpen = false;
  }

  async function handleDelete() {
    try {
      let res = await commands.deleteClient(client.id);
      if (res.status === "ok") {
        appStore.removeClient(client.id);
        await goto("/clients");
      } else {
        // TODO: show error to user
        alert(res.error);
      }
    } catch (err) {
      // TODO: show error to user
      alert(`Critical Tauri panic or IPC error: ${err}`);
    }
  }
</script>

<PageLayout>
  {#snippet header()}
    <div class="h-full px-4 flex items-center justify-between">
      <Button variant="secondary" onclick={() => goto("/clients")}>
        <ChevronLeftIcon /> Список
      </Button>
      <ButtonGroup>
        <Sheet.Root bind:open={isSheetOpen}>
          <Sheet.Trigger class={buttonVariants({ variant: "secondary" })}>
            <SquarePenIcon /> Редагувати
          </Sheet.Trigger>

          <Sheet.Content
            side="right"
            showCloseButton={false}
            onEscapeKeydown={(e) => e.preventDefault()}
            onInteractOutside={(e) => e.preventDefault()}
          >
            <Sheet.Header>
              <Sheet.Title>Редагувати клієнта</Sheet.Title>
              <Sheet.Description
                >Внесіть зміни до профілю клієнта тут.</Sheet.Description
              >
            </Sheet.Header>

            <form
              use:enhance
              id="edit-client-form"
              class="flex flex-col p-6 gap-6"
            >
              <Form.Field {form} name="name">
                <Form.Control>
                  {#snippet children({ props })}
                    <Label>Ім'я</Label>
                    <Input {...props} bind:value={$formData.name} />
                  {/snippet}
                </Form.Control>
                <Form.FieldErrors />
              </Form.Field>

              <Form.Field {form} name="phone">
                <Form.Control>
                  {#snippet children({ props })}
                    <Label>Мобільний номер</Label>
                    <Input {...props} bind:value={$formData.phone} />
                  {/snippet}
                </Form.Control>
                <Form.FieldErrors />
              </Form.Field>
            </form>

            <Sheet.Footer>
              <Button
                type="submit"
                disabled={$submitting}
                form="edit-client-form"
              >
                {#if $submitting}<Spinner />{/if}
                Зберегти зміни
              </Button>
              <Button
                variant="outline"
                disabled={$submitting}
                onclick={handleCancel}
              >
                Відмінити та закрити
              </Button>
            </Sheet.Footer>
          </Sheet.Content>
        </Sheet.Root>
        <AlertDialog.Root>
          <AlertDialog.Trigger
            class={buttonVariants({ variant: "destructive" })}
          >
            <TrashIcon /> Видалити
          </AlertDialog.Trigger>

          <AlertDialog.Content
            onEscapeKeydown={(e) => {
              if (isDeleting) e.preventDefault();
            }}
          >
            <AlertDialog.Header>
              <AlertDialog.Title>Підтвердіть видалення</AlertDialog.Title>
              <AlertDialog.Description>
                Цю дію неможливо відмінити. Усі пов'язані з цим клієнтом дані
                будуть видалені з бази даних.
              </AlertDialog.Description>
            </AlertDialog.Header>
            <AlertDialog.Footer>
              <AlertDialog.Cancel disabled={isDeleting}
                >Відмінити</AlertDialog.Cancel
              >
              <AlertDialog.Action disabled={isDeleting} onclick={handleDelete}>
                {#if isDeleting}<Spinner />{/if}
                Видалити
              </AlertDialog.Action>
            </AlertDialog.Footer>
          </AlertDialog.Content>
        </AlertDialog.Root>
      </ButtonGroup>
    </div>
  {/snippet}
  <div class="p-4">
    <div class="border p-4 rounded">
      <div class="flex items-center justify-between">
        <h1 class="text-2xl font-semibold">{client?.name}</h1>
        <p class="text-muted-foreground">ID: {client?.id}</p>
      </div>
      <div class="flex items-center justify-between mt-2">
        <p class="flex items-center gap-2">
          {client?.phone}
        </p>
        <p class="flex items-center gap-2 text-muted-foreground">
          {dateFormatter.format(new Date(client?.created_at))}
        </p>
      </div>
    </div>
    <div class="border rounded p-4 mt-4">Cars here...</div>
  </div>
</PageLayout>

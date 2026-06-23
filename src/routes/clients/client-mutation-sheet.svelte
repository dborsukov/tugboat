<script lang="ts">
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import { commands } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import { Button } from "$lib/components/ui/button";
  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Sheet from "$lib/components/ui/sheet";
  import { Spinner } from "$lib/components/ui/spinner";

  interface Props {
    open: boolean;
    client?: any | null;
  }

  let { open = $bindable(false), client = null }: Props = $props();

  const isEdit = $derived(!!client);
  const titleText = $derived(
    isEdit ? "Редагувати клієнта" : "Створити клієнта",
  );
  const descText = $derived(
    isEdit ? "Змініть дані клієнта тут." : "Внесіть дані нового клієнта тут.",
  );
  const formID = "client-mutation-form";

  export const clientSchema = z.object({
    name: z.string().nonempty("Ім'я не може бути пустим"),
    phone: z.string().nonempty("Мобільний номер не може бути пустим"),
  });

  const form = superForm(defaults(zod4(clientSchema)), {
    SPA: true,
    validators: zod4Client(clientSchema),
    validationMethod: "oninput",
    resetForm: false,
    id: formID,
    async onUpdate({ form: nativeForm }) {
      if (!nativeForm.valid) return;

      const payload = {
        name: $formData.name,
        phone: $formData.phone,
      };

      try {
        if (isEdit) {
          let res = await commands.updateClient(client.id, payload);
          if (res.status === "ok") {
            appStore.updateClient(res.data);
          } else {
            // TODO: show error to user
            alert(res.error);
          }
        } else {
          let res = await commands.createClient(payload);
          if (res.status === "ok") {
            appStore.createClient(res.data);
          } else {
            // TODO: show error to user
            alert(res.error);
          }
        }
        open = false;
        form.reset();
      } catch (err) {
        alert(`Critical IPC error: ${err}`);
      }
    },
  });

  const { form: formData, errors, enhance, submitting } = form;

  $effect(() => {
    if (open) {
      if (client) {
        $formData.name = client.name;
        $formData.phone = client.phone;
      } else {
        form.reset();
      }
    }
  });

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

<script lang="ts">
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import UserPlusIcon from "@lucide/svelte/icons/user-plus";

  import { commands } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import { Button, buttonVariants } from "$lib/components/ui/button";
  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Pagination from "$lib/components/ui/pagination/index.js";
  import * as Sheet from "$lib/components/ui/sheet";
  import { Spinner } from "$lib/components/ui/spinner";

  import PageLayout from "$lib/components/page-layout.svelte";

  import ClientCard from "./client-card.svelte";

  let currentPage = $state(1);
  let perPage = $state(15);
  let searchQuery = $state("");
  let isSheetOpen = $state(false);

  let filteredClients = $derived(
    appStore.clients.filter((client) => {
      const query = searchQuery.trim().toLowerCase();
      if (!query) return true;

      const matchesName = client.name.toLowerCase().includes(query);
      const matchesPhone = client.phone!.includes(query);

      return matchesName || matchesPhone;
    }),
  );

  let paginatedClients = $derived(
    filteredClients.slice((currentPage - 1) * perPage, currentPage * perPage),
  );

  function handleSearchInput() {
    currentPage = 1;
  }

  export const userSchema = z.object({
    name: z.string().nonempty("Ім'я не може бути пустим"),
    phone: z.string().nonempty("Мобільний номер не може бути пустим"),
  });

  const form = superForm(defaults(zod4(userSchema)), {
    SPA: true,
    validators: zod4Client(userSchema),
    validationMethod: "oninput",
    resetForm: false,
    async onUpdate({ form }) {
      if (!form.valid) return;
      try {
        let res = await commands.createClient({
          name: form.data.name,
          phone: form.data.phone,
        });
        if (res.status === "ok") {
          let new_client = res.data;
          appStore.clients.push(new_client);
          form.data.name = "";
          form.data.phone = "";
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
</script>

<PageLayout>
  {#snippet header()}
    <div class="h-full px-4 flex items-center gap-2">
      <!-- TODO: implement fuzzy search with highlight -->
      <Input
        placeholder="Пошук за ім'ям або телефоном..."
        class="w-full"
        bind:value={searchQuery}
        oninput={handleSearchInput}
      />
      <Sheet.Root bind:open={isSheetOpen}>
        <Sheet.Trigger class={buttonVariants({ variant: "default" })}>
          <UserPlusIcon /> Створити
        </Sheet.Trigger>

        <Sheet.Content
          side="right"
          showCloseButton={false}
          onEscapeKeydown={(e) => e.preventDefault()}
          onInteractOutside={(e) => e.preventDefault()}
        >
          <Sheet.Header>
            <Sheet.Title>Створити клієнта</Sheet.Title>
            <Sheet.Description
              >Внесіть данні нового клієнта тут.</Sheet.Description
            >
          </Sheet.Header>

          <form
            use:enhance
            id="create-client-form"
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
              form="create-client-form"
            >
              {#if $submitting}<Spinner />{/if}
              Створити
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
    </div>
  {/snippet}
  {#if paginatedClients.length === 0}
    <!-- TODO: center this or replace with shadcn's Empty -->
    <div class="flex h-40 items-center justify-center text-muted-foreground">
      Клієнтів не знайдено
    </div>
  {:else}
    <div class="flex flex-wrap gap-4 p-4">
      {#each paginatedClients as client}
        <a href="/clients/{client.id}" class="flex-1 min-w-max">
          <ClientCard
            name={client.name}
            phone={client.phone}
            date={new Date(client.created_at)}
          />
        </a>
      {/each}
    </div>
  {/if}
  {#snippet footer()}
    <div class="p-4 flex items-center justify-center">
      <Pagination.Root
        count={appStore.clients.length}
        {perPage}
        bind:page={currentPage}
      >
        {#snippet children({ pages, currentPage })}
          <Pagination.Content>
            <Pagination.Item>
              <Pagination.PrevButton />
            </Pagination.Item>

            {#each pages as page (page.key)}
              {#if page.type === "ellipsis"}
                <Pagination.Item>
                  <Pagination.Ellipsis />
                </Pagination.Item>
              {:else}
                <Pagination.Item>
                  <Pagination.Link {page} isActive={currentPage === page.value}>
                    {page.value}
                  </Pagination.Link>
                </Pagination.Item>
              {/if}
            {/each}

            <Pagination.Item>
              <Pagination.NextButton />
            </Pagination.Item>
          </Pagination.Content>
        {/snippet}
      </Pagination.Root>
    </div>
  {/snippet}
</PageLayout>

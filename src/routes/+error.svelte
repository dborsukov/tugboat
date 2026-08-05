<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";

  import { Button } from "$lib/components/ui/button";

  function getErrorInfo(status: number, message?: string) {
    if (status === 404) {
      return {
        title: "Сторінку не знайдено",
        description:
          message || "Вказаний шлях не існує або посилання застаріло.",
      };
    }
    if (status === 403) {
      return {
        title: "Доступ обмежено",
        description:
          message || "У вас немає дозволу на перегляд цієї сторінки.",
      };
    }
    if (status >= 500) {
      return {
        title: "Серверна помилка",
        description:
          message || "Сталася помилка на сервері. Спробуйте оновити сторінку.",
      };
    }
    return {
      title: `Помилка (${status})`,
      description: message || "Виникла непередбачувана помилка.",
    };
  }

  const info = $derived(getErrorInfo(page.status, page.error?.message));
</script>

<main
  class="flex h-full min-h-[70vh] flex-col items-center justify-center text-center"
>
  <div class="mx-auto flex max-w-md flex-col items-center gap-4">
    <div class="space-y-1.5">
      <h1 class="text-2xl font-semibold tracking-tight">
        {info.title} ({page.status})
      </h1>
      <p class="text-sm text-muted-foreground">
        {info.description}
      </p>
    </div>

    <Button
      variant="link"
      onclick={() => goto("/")}
      class="hover:cursor-pointer">Повернутись на головну</Button
    >
  </div>
</main>

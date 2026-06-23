<script lang="ts">
  import { page } from "$app/state";
</script>

<div
  class="flex flex-col gap-2 items-center justify-center h-full p-8 text-center bg-background text-foreground"
>
  <h1 class="text-2xl font-bold tracking-tight">
    Сторінку не знайдено ({page.status})
  </h1>
  <p class="text-sm text-muted-foreground max-w-xs">
    Вказаний шлях не існує або інтерфейс посилання застарів.
  </p>
  <a href="/" class="text-sm font-medium text-primary underline">
    Повернутися на головну
  </a>
</div>

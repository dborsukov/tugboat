import { toast } from "svelte-sonner";

import { copyText } from "$lib/hooks/use-clipboard.svelte";

/** Copy a labeled value and report the result with the app's standard toast. */
export async function copyValue(value: string, label: string) {
  const status = await copyText(value);
  if (status === "success") {
    toast.success(`${label} скопійовано`, { position: "bottom-right" });
  } else {
    toast.error("Не вдалося скопіювати", {
      description: label,
      position: "bottom-right",
    });
  }
}

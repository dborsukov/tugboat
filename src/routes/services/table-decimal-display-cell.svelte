<script lang="ts">
  import { NumericText } from "svelte-number-format";

  let { value } = $props();
</script>

<div class="text-center">
  <NumericText
    {value}
    locale="uk-UA"
    options={{
      precision: 1,
    }}
  />
</div>

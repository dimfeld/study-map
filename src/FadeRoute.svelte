<script lang="typescript">
  import { fade } from 'svelte/transition';
  import { Route } from 'tinro';

  export let delay = 200;
</script>

<Route {...$$restProps} let:meta>
  <div transition:fade={{ delay }}>
    <slot {meta} />
  </div>
</Route>

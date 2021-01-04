<script lang="typescript">
  import { fade } from 'svelte/transition';
  import { Route } from 'tinro';

  export let duration = 100;
</script>

<Route {...$$restProps} let:meta>
  <div transition:fade|local={{ duration }}>
    <slot {meta} />
  </div>
</Route>

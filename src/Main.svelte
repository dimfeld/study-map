<script lang="typescript">
  import type { BookRoot } from './types';
  import BookOverview from './BookOverview.svelte';
  import { Route } from 'tinro';
  import FadeRoute from './FadeRoute.svelte';
  import BookRangeText from './BookRangeText.svelte';

  export let book: BookRoot;
</script>

<style>
  /* Force all pages into the same place in the grid, so that they occupy the same place
  during the fade. */
  #main > :global(div) {
    grid-column: 1;
    grid-row: 1;
  }
</style>

<div id="main" class="w-full h-full grid grid-cols-1 grid-rows-1 overflow-auto">
  <FadeRoute path="/">
    <BookOverview {book} />
  </FadeRoute>

  <FadeRoute path="/passage/:start/:end" let:meta>
    <button type="button"><a href="/">Back to Overview</a></button>
    <BookRangeText
      start={meta.params.start.split(',').map((x) => +x)}
      end={meta.params.end.split(',').map((x) => +x)}
      {book} />
  </FadeRoute>
</div>

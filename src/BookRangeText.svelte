<script lang="typescript">
  import { getContext } from 'svelte';
  import type { Chunk } from './chunks';
  import type { BookRoot } from './types';
  import type { Writable } from 'svelte/store';
  import type { ResultTree } from './result_tree';

  export let start: number[];
  export let end: number[];
  export let book: BookRoot;

  const results = getContext<Writable<ResultTree>>('search-results');

  let textChunks = [];

  $: {
    let path = start.slice();

    textChunks = [];
    while (path[0] <= end[0]) {
      textChunks.push([...path]);

      path[1]++;
      if (path[1] >= book.children[path[0]].children.length) {
        path = [path[0] + 1, 0];
      } else if (path[0] === end[0] && path[1] > end[1]) {
        break;
      }
    }
  }
</script>

{#each textChunks as path}
  <section>
    <h1 class="text-xl text-gray-600 font-medium">
      {book.children[path[0]].name}
      {path[1] + 1}
    </h1>
  </section>
{/each}

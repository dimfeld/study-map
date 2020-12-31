import flatten from 'just-flatten-it';
import { SearchResult } from './types';

export function resultTree(results: SearchResult[], maxDepth: number) {
  let root = [];

  for (let result of results) {
    let path = [result.l0, result.l1, result.l2].slice(0, maxDepth + 1);

    let node = root;
    for (let pathComponent of path.slice(0, path.length - 1)) {
      let child = node[pathComponent];
      if (!child) {
        child = node[pathComponent] = [];
      }

      node = child;
    }

    let lastPath = path[path.length - 1];
    let resultsList = node[lastPath];
    if (resultsList) {
      resultsList.push(result);
    } else {
      node[lastPath] = [result];
    }
  }

  return {
    results,
    subtree: (path: number[]) => {
      let node = root;
      for (let p of path) {
        node = node[p];
        if (!node) {
          return [];
        }
      }

      console.log({ path, node });
      return flatten(node).filter(Boolean) as SearchResult[];
    },
  };
}

export const emptyResultTree: ResultTree = {
  results: [],
  subtree: () => [],
};

export type ResultTree = ReturnType<typeof resultTree>;

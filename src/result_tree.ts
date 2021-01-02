import flatten from 'just-flatten-it';
import { SearchResult } from './types';

export function resultTree(results: SearchResult[], maxDepth: number) {
  let root: SearchResult[][][] = [];

  for (let result of results) {
    let path = [result.l0, result.l1, result.l2].slice(0, maxDepth + 1);

    let node: any = root;
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
    range: (startPath: number[], endPath: number[]) => {
      let path = startPath;
      let output = [];

      let l0Node: SearchResult[][];
      while (path[0] <= endPath[0]) {
        l0Node = root[path[0]] || [];
        if (path[0] === endPath[0]) {
          l0Node = l0Node.slice(path[1], endPath[1] + 1);
        } else if (path[1] > 0) {
          l0Node = l0Node.slice(path[1]);
        }

        output.push(...l0Node.filter(Boolean));
        path = [path[0] + 1, 0];
      }

      console.log({ startPath, endPath, output });
      return flatten(output) as SearchResult[];
    },
  };
}

export const emptyResultTree: ResultTree = {
  results: [],
  range: () => [],
};

export type ResultTree = ReturnType<typeof resultTree>;

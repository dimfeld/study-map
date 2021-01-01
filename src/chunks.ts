import { BookRoot, BookDataNode } from './types';
import Heapify from 'heapify';
import sorter from 'sorters';

function describeRange(
  book: BookRoot,
  startRange: number[],
  endRange: number[]
) {
  let startBook = book.children[startRange[0]] as BookDataNode;
  let endBook = book.children[endRange[0]] as BookDataNode;

  let startChapter = startRange[1] + 1;
  let endChapter = endRange[1] + 1;

  if (startBook === endBook) {
    if (startChapter === 1 && endChapter === startBook.children.length) {
      return startBook.name;
    } else if (startChapter === endChapter) {
      return `${startBook.name} ${startChapter}`;
    } else {
      return `${startBook.name} ${startChapter}-${endChapter}`;
    }
  } else if (startChapter === 1 && endChapter === endBook.children.length) {
    return `${startBook.name} - ${endBook.name}`;
  } else {
    return `${startBook.name} ${startChapter} - ${endBook.name} ${endChapter}`;
  }
}

/** Split a book into chunks, enforcing that every chunk is completely within a single L0-chunk
 * (i.e. a Bible book) when possible. */
export function l0BoundaryChunks(numElements: number, book: BookRoot) {
  if (book.children.length < numElements) {
    return splitL0Children(numElements, book);
  } else if (book.children.length > numElements) {
    // There are more L0 children than there are elements, so we have to merge the smaller ones together.
    return mergeL0Children(numElements, book);
  } else {
    // An exact match, so each range maps to exactly one L0 child.
    return book.children.map((child: BookDataNode, i) => {
      return {
        start: [i, 0],
        end: [i, child.children.length - 1],
        title: child.name,
      };
    });
  }
}

/** We need more elements than we have L0 children, so split the largest children into multiple ranges */
function splitL0Children(desiredElements: number, book: BookRoot) {
  let children = book.children.map((c) => ({
    totalLength: c.len,
    chunks: 1,
  }));

  // Reverse chunk size, so that the minheap actually gives us the largest one.
  const childPriority = (c) => 0x7fffffff - Math.ceil(c.totalLength / c.chunks);

  let keys = children.map((_c, i) => i);
  let priorities = children.map(childPriority);
  let heap = new Heapify(children.length, keys, priorities);

  let numElements = children.length;

  // Increase the chunk count of the largest element, and repeat until we have the desired
  // number of chunks.
  while (numElements < desiredElements) {
    let index = heap.pop();
    let child = children[index];
    child.chunks++;
    numElements++;
    heap.push(index, childPriority(child));
  }

  return children.flatMap(({ chunks }, l0) => {
    let child = book.children[l0] as BookDataNode;
    let chunkSize = child.children.length / chunks;
    return Array.from({ length: chunks }, (_, i) => {
      let start = [l0, Math.round(chunkSize * i)];
      let end = [l0, Math.round(chunkSize * (i + 1)) - 1];
      return {
        start,
        end,
        title: describeRange(book, start, end),
      };
    });
  });
}

function mergeL0Children(desiredElements: number, book: BookRoot) {
  let chunks = book.children
    .map((c, i) => ({
      totalLength: c.len,
      l0: [i, i],
      originalIndex: i,
    }))
    .sort(sorter('totalLength'));

  let chunkMap = new Map(chunks.map((c) => [c.l0[0], c]));

  let numElements = chunks.length;
  while (numElements > desiredElements) {
    if (chunks[0].totalLength > chunks[1].totalLength) {
      // Sort the array again, only if the first element is no longer the smallest.
      chunks.sort(sorter('totalLength'));
    }

    let chunk = chunks.shift();

    let prevChunk = chunkMap.get(chunk.l0[0] - 1);
    let nextChunk = chunkMap.get(chunk.l0[1] + 1);

    let prevSize = prevChunk?.totalLength ?? Infinity;
    let nextSize = nextChunk?.totalLength ?? Infinity;

    let mergeInto;
    if (prevSize < nextSize) {
      mergeInto = prevChunk;
      mergeInto.l0[1] = chunk.l0[1];
    } else {
      mergeInto = nextChunk;
      mergeInto.l0[0] = chunk.l0[0];
    }

    mergeInto.totalLength += chunk.totalLength;

    // chunk map lookups for the original chunk should now go to the merged chunk.
    chunkMap.set(chunk.originalIndex, mergeInto);
    numElements--;
  }

  chunks.sort(sorter((c) => c.l0[0]));

  return chunks.map((c) => {
    let start = [c.l0[0], 0];
    let end = [c.l0[1], book.children[c.l0[1]].children.length - 1];
    return {
      start,
      end,
      title: describeRange(book, start, end),
    };
  });
}

/** Split a book into as even chunks as possible, at an L1 granularity (Bible chapters). This
 * was an earlier idea I had but it ended up kind of weird when chunks crossed book boundaries.
 * Keeping it here for reference and in case it actually ends up working better for some other
 * type of text.
 */
export function evenChunks(numElements: number, book: BookRoot) {
  let idealLengthPerElement = book.len / numElements;
  let ranges = new Array(numElements);
  let nextPath = [0, 0];

  function getNextPassage() {
    let [l0, l1] = nextPath;
    if (l0 === book.children.length) {
      return { passage: null, path: null };
    }

    let passage = (book.children[l0] as BookDataNode).children[l1];
    let currentPath = nextPath.slice();

    l1++;
    if (l1 === (book.children[l0] as BookDataNode).children.length) {
      // Wrapping around to the next book.
      l1 = 0;
      l0++;
    }

    nextPath = [l0, l1];

    return { path: currentPath, passage };
  }

  function ungetPassage() {
    if (nextPath[1] === 0) {
      nextPath[0]--;
      nextPath[1] = book.children[nextPath[0]].children.length - 1;
    } else {
      nextPath[1]--;
    }
  }

  let totalLengthConsumed = 0;
  for (let i = 0; i < numElements; ++i) {
    let startRange = nextPath.slice();
    let endRange = null;
    let currentLength = 0;

    // Set a desired length that brings us closest to the average
    let idealDelta = totalLengthConsumed % idealLengthPerElement;
    let desiredLength;
    if (idealDelta > idealLengthPerElement / 2) {
      desiredLength =
        idealLengthPerElement + (idealLengthPerElement - idealDelta);
    } else {
      desiredLength = idealLengthPerElement - idealDelta;
    }

    // desiredLength = idealLengthPerElement;

    while (currentLength < desiredLength) {
      let { passage, path } = getNextPassage();
      if (!passage) {
        break;
      }

      // If this one goes over the limit, then take it or not depending on what brings us closer to the desired length for this section.
      let newLength = currentLength + passage.len;
      if (newLength > desiredLength) {
        let deltaWith = newLength - desiredLength;
        let deltaWithout = desiredLength - currentLength;

        if (deltaWith > deltaWithout && currentLength > 0) {
          // It brings us closer to the desired value to not use this, so put it back.
          ungetPassage();
        } else {
          // Use it and then finish this range.
          endRange = path;
          currentLength = newLength;
        }

        totalLengthConsumed += currentLength;
        break;
      } else {
        endRange = path;
        currentLength = newLength;
      }
    }

    ranges[i] = {
      start: startRange,
      end: endRange,
      title: describeRange(book, startRange, endRange),
    };
  }

  return ranges;
}

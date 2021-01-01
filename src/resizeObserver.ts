const observed = new WeakMap();

const observer = new ResizeObserver((entries) => {
  for (let entry of entries) {
    let cb = observed.get(entry.target);
    cb?.(entry);
  }
});

export default function (node, cb) {
  observed.set(node, cb);
  observer.observe(node, { box: 'border-box' });

  return {
    destroy() {
      observed.delete(node);
      observer.unobserve(node);
    },
  };
}

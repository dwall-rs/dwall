export function mergeRefs(
  ...refs: Array<Element | ((el: Element) => void) | undefined>
) {
  return (el: Element) => {
    for (const ref of refs) {
      if (typeof ref === "function") ref(el);
    }
  };
}

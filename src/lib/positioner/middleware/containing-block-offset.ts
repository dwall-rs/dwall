import type { Middleware } from "../types";

const TRANSFORM_STYLE_PROPS = [
  "transform",
  "perspective",
  "filter",
  "backdropFilter",
] as const;

function establishesContainingBlock(style: CSSStyleDeclaration): boolean {
  for (const prop of TRANSFORM_STYLE_PROPS) {
    const value = style[prop as keyof CSSStyleDeclaration] as
      | string
      | undefined;
    if (value && value !== "none") return true;
  }
  const willChange = style.willChange || "";
  if (/transform|perspective|filter/.test(willChange)) return true;
  const contain = style.contain || "";
  if (/paint|layout|strict|content/.test(contain)) return true;
  return false;
}

/**
 * Fixes the systematic x/y offset caused when an ancestor sets CSS properties
 * like transform / filter / perspective / backdrop-filter / will-change /
 * contain, which redefine the containing block of position:fixed (and some-
 * times position:absolute) from the "viewport" to that ancestor itself.
 *
 * Typical symptom: positioning works fine in a demo page, but in a real
 * project (especially apps that use transform heavily for drag/zoom/panel
 * animations — design tools, kanban boards, canvas editors) the whole
 * floating layer is offset in one direction while relative relationships
 * (e.g. the arrow shifting with the content) stay intact — the signature of
 * "the coordinate system was redefined by an ancestor", not a wrong algorithm.
 *
 * Principle: walk up from the floating's actual DOM parent, find the first
 * ancestor that redefines the containing block, derive that "accidental
 * containing block"'s viewport offset from its getBoundingClientRect(),
 * and subtract it from the final coordinates to cancel the shift.
 *
 * Limitation: only pure translation offsets are handled (e.g. translate,
 * panel expand animations, the common GPU-acceleration translateZ(0)). If
 * that ancestor also applies scale/rotate, no matrix conversion is done and
 * the position will still be off — the more thorough fix is to move the
 * Portal mount point out from under that ancestor (e.g. manually switch to a
 * truly unaffected document.body rather than a deeply nested app container).
 *
 * Put it last in the middleware array — it corrects the coordinates that
 * finally land in the DOM; preceding offset/flip/shift/size/arrow all compute
 * in a unified viewport coordinate system, so only this final step needs it.
 */
export function containingBlockOffset(): Middleware {
  return {
    name: "containingBlockOffset",
    fn(state) {
      const floatingEl = state.elements.floating;
      let node: Element | null = floatingEl.parentElement;
      let offsetX = 0;
      let offsetY = 0;
      let foundAncestor: Element | null = null;

      while (node && node !== document.documentElement) {
        const style = getComputedStyle(node);
        if (establishesContainingBlock(style)) {
          const rect = node.getBoundingClientRect();
          offsetX = rect.x;
          offsetY = rect.y;
          foundAncestor = node;
          break;
        }
        node = node.parentElement;
      }

      if (!foundAncestor || (offsetX === 0 && offsetY === 0)) {
        return { data: { ancestor: null } };
      }

      return {
        x: state.x - offsetX,
        y: state.y - offsetY,
        data: { ancestor: foundAncestor, offsetX, offsetY },
      };
    },
  };
}

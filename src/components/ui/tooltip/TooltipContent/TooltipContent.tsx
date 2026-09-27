import { Show, onCleanup, splitProps } from "solid-js";
import { Portal } from "solid-js/web";
import { useTooltipContent } from "./useTooltipContent";
import { TooltipContentContext } from "./TooltipContent.context";
import { getTransformOrigin } from "./TooltipContent.utils";
import { getSide, getAlignment } from "~/lib";
import type { TooltipContentProps } from "./TooltipContent.types";
import { clsx } from "~/utils";

/**
 * Tooltip's floating content: portals into the body, positioned via
 * position:fixed.
 *
 * The content itself is hoverable by default (not pointer-events:none):
 * moving into it cancels the pending close timer (ctx.keepOpen); only leaving
 * it runs the closeDelay close — so users can move from the trigger onto the
 * tooltip to select its text instead of losing it to "closes as soon as you
 * leave the trigger". If your content is simple, wrap it in your own layer
 * with pointer-events: none.
 */
export function TooltipContent(props: TooltipContentProps) {
  const {
    ctx,
    pos,
    mounted,
    animationState,
    setArrowElement,
    setContentElement,
    slideStyle,
  } = useTooltipContent(() => props);

  // Pull out only the fields handled internally; the rest (style, data-*,
  // onXxx, any other native prop) passes through — same as TooltipTrigger.
  const [local, rest] = splitProps(props, [
    "side",
    "align",
    "sideOffset",
    "alignOffset",
    "collisionPadding",
    "class",
    "style",
    "children",
  ]);

  return (
    <Show when={mounted()}>
      <Portal>
        {/*
          Outer layer: positioning only (position:fixed comes from
          floatingStyles), no overflow, no animation — its transform is
          already used for positioning (translate(x,y)), so another animation
          driving transform would conflict; all animations live below.
        */}
        <div
          ref={(el) => {
            ctx.setFloating(el);
            // Solid's ref callback fires only once on element creation, not
            // again with undefined on removal (that is React's convention,
            // not Solid's). Without manually clearing, ctx.floating() keeps
            // pointing at a "zombie node" already removed from the DOM until
            // the next open — if read elsewhere meanwhile (e.g. TooltipGroup
            // preemption reading the rect), getBoundingClientRect() per spec
            // returns an all-zero rect for a detached element, giving an
            // obviously wrong "slides in from (0,0)" look.
            onCleanup(() => ctx.setFloating(undefined));
          }}
          data-placement={pos.placement()}
          style={{
            ...pos.floatingStyles(),
            "z-index": 1000,
            opacity: pos.isPositioned() ? 1 : 0,
          }}
        >
          {/*
            Slide layer: on a <TooltipGroup> preemption hit, translates the whole
            tooltip (content + arrow) from the old tooltip's position via FLIP
            (see the slideOffset computation in useTooltipContent). Normally
            slideStyle() is an empty object, so this layer is effectively absent.

            This layer must sit between the "positioning layer" and the
            "style/animation layer", not anywhere else: it uses transform while
            sliding, which would clash with translate(x,y) if it shared an
            element with the positioning layer (same reason as above). Nor can
            it sit below/inside the style layer (the overflow:auto one), since
            the arrow's position:absolute deliberately skips that layer and
            references "the nearest positioned ancestor" (the design that fixed
            the arrow poking out and triggering a scrollbar). Were its
            transform inside the style layer, the arrow's reference would become
            the slide layer instead of the outer one — still visually correct
            (their boxes nearly coincide), but to keep the positioning reference
            unambiguous and avoid the "technically vs. visually who" mental
            load, the slide layer sits between the outer and style layers,
            preserving the chain's clear order.
          */}
          <div style={slideStyle()}>
            {/*
              Inner layer: real visual styling + enter/exit animation.
              data-state follows animationState (not isVisible/mounted):
              - the moment isVisible turns false, animationState turns closed
                right away, triggering the exit-animation class;
              - when isVisible turns true, animationState turns open a frame
                later, giving the arrow (TooltipArrow) time to mount, register
                with the arrow middleware, and compute its position offset
                (see the comments in useTooltipContent);
              - mounted turns false after animationState does (see the
                Presence logic below), giving the exit animation time to end.

              The data-[state=open]/data-[state=closed] classes rely on utility
              classes from the tailwindcss-animate plugin
              (animate-in/animate-out/fade-in-0/zoom-in-95, …). If your project
              doesn't have it, `npm install tailwindcss-animate` and add the
              plugin to your tailwind config; otherwise swap these
              data-[state=...] classes for your own @keyframes + classes — the
              Presence logic (delayed unmount via mounted) doesn't depend on a
              specific animation implementation.

              The arrow (TooltipArrow) doesn't rely on the implicit "child of
              this element passively inheriting its transform/opacity
              animation" — it reads the same animationState from context and
              plays its own copy, both triggered in sync by one signal.

              data-side/data-align are split from the effective placement,
              driving the direction-aware slide-in classes above and giving
              transform-origin its basis — the scale animation anchors on "the
              edge closest to the trigger" (see getTransformOrigin in
              TooltipContent.utils.ts) rather than the default center, so the
              entrance looks like it "grows out of the trigger", close to
              shadcn's actual look.

              Deliberately no overflow:auto / max-height — tooltips don't
              support scrolling when content is too long. Width is capped by
              the static max-w-xs class below so text wraps naturally; height
              is unrestricted. This is not only to match shadcn/Radix (their
              tooltips don't scroll either), it also avoids a real pitfall:
              overflow:auto combined with the zoom-in-95 animation here (which
              adds a non-none transform, making this element a new containing
              block for children per the CSS spec) once made the arrow's
              overhang be misjudged as "scrollable overflow" — on open, a
              scrollbar flashed and clipped the arrow, then the scrollbar
              vanished and the arrow returned when the animation ended.
              Dropping scrolling removes this coupling at the root.
            */}
            <div
              ref={setContentElement}
              id={ctx.contentId}
              role="tooltip"
              data-state={animationState()}
              data-side={getSide(pos.placement())}
              data-align={getAlignment(pos.placement()) ?? "center"}
              onMouseEnter={ctx.keepOpen}
              onMouseLeave={ctx.requestClose}
              style={{
                "transform-origin": getTransformOrigin(pos.placement()),
                ...(typeof local.style === "object" ? local.style : undefined),
              }}
              class={clsx(
                "inline-flex w-fit max-w-xs items-center gap-1.5 rounded-md bg-foreground px-3 py-1.5 text-xs text-background",
                "has-data-[slot=kbd]:pr-1.5",
                "data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95",
                "data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95",
                // Direction-aware slide-in: not just a fade/zoom in place, it
                // also drifts in from the side opposite the trigger — e.g.
                // placement top (content above the trigger) slides up from 8px
                // below (slide-in-from-bottom-2), looking like it "emerges"
                // from the trigger rather than appearing from nowhere. Only on
                // open — matching shadcn, whose exit has no slide-out.
                "data-[side=top]:slide-in-from-bottom-2",
                "data-[side=bottom]:slide-in-from-top-2",
                "data-[side=left]:slide-in-from-right-2",
                "data-[side=right]:slide-in-from-left-2",
                local.class,
              )}
              {...rest}
            >
              <TooltipContentContext.Provider
                value={{
                  middlewareData: pos.middlewareData,
                  reference: ctx.reference,
                  placement: pos.placement,
                  setArrowElement,
                  animationState,
                }}
              >
                {local.children}
              </TooltipContentContext.Provider>
            </div>
          </div>
        </div>
      </Portal>
    </Show>
  );
}

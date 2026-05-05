export function computePosition(
  trigger: HTMLElement,
  content: HTMLElement,
  side: "top" | "bottom" | "left" | "right",
  align: "start" | "center" | "end",
  alignOffset: number,
  sideOffset: number,
) {
  const rect = trigger.getBoundingClientRect();
  const contentWidth = content.offsetWidth;
  const contentHeight = content.offsetHeight;
  const vw = window.innerWidth;
  const vh = window.innerHeight;
  const MARGIN = 4;

  let top = 0;
  let left = 0;
  let resolvedSide = side;
  let resolvedAlign = align;

  // --- Flip：优先检测对侧是否有足够空间，再决定最终 side ---
  switch (side) {
    case "top":
      if (
        rect.top < contentHeight + sideOffset &&
        rect.bottom + sideOffset + contentHeight <= vh
      ) {
        resolvedSide = "bottom";
      }
      break;
    case "bottom":
      if (
        rect.bottom + sideOffset + contentHeight > vh &&
        rect.top - sideOffset - contentHeight >= 0
      ) {
        resolvedSide = "top";
      }
      break;
    case "left":
      if (
        rect.left < contentWidth + sideOffset &&
        rect.right + sideOffset + contentWidth <= vw
      ) {
        resolvedSide = "right";
      }
      break;
    case "right":
      if (
        rect.right + sideOffset + contentWidth > vw &&
        rect.left - sideOffset - contentWidth >= 0
      ) {
        resolvedSide = "left";
      }
      break;
  }

  // --- 主轴位置（基于 resolvedSide）---
  switch (resolvedSide) {
    case "top":
      top = rect.top - contentHeight - sideOffset;
      break;
    case "bottom":
      top = rect.bottom + sideOffset;
      break;
    case "left":
      left = rect.left - contentWidth - sideOffset;
      break;
    case "right":
      left = rect.right + sideOffset;
      break;
  }

  // --- 交叉轴位置（align）---
  if (resolvedSide === "top" || resolvedSide === "bottom") {
    switch (align) {
      case "start":
        left = rect.left + alignOffset;
        break;
      case "center":
        left = rect.left + rect.width / 2 - contentWidth / 2 + alignOffset;
        break;
      case "end":
        left = rect.right - contentWidth - alignOffset;
        break;
    }
  } else {
    switch (align) {
      case "start":
        top = rect.top + alignOffset;
        break;
      case "center":
        top = rect.top + rect.height / 2 - contentHeight / 2 + alignOffset;
        break;
      case "end":
        top = rect.bottom - contentHeight - alignOffset;
        break;
    }
  }

  // --- Shift：贴边平移，并更新 resolvedAlign ---
  if (resolvedSide === "top" || resolvedSide === "bottom") {
    if (left < MARGIN) {
      left = MARGIN;
      resolvedAlign = "start";
    } else if (left + contentWidth > vw - MARGIN) {
      left = vw - contentWidth - MARGIN;
      resolvedAlign = "end";
    }
  } else {
    if (top < MARGIN) {
      top = MARGIN;
      resolvedAlign = "start";
    } else if (top + contentHeight > vh - MARGIN) {
      top = vh - contentHeight - MARGIN;
      resolvedAlign = "end";
    }
  }

  return { top, left, side: resolvedSide, align: resolvedAlign };
}

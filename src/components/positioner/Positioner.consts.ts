/** 对侧方向 */
export const OPPOSITE: Record<string, "top" | "bottom" | "left" | "right"> = {
  top: "bottom",
  bottom: "top",
  left: "right",
  right: "left",
};

// 初始挂载阶段：opacity:0 + position:fixed，让浏览器完成完整 layout 再测量。
// 测量完成后切换为 opacity:1 + 真实坐标。
// 注意：不用 visibility:hidden，因为 visibility 影响 offsetTop 在某些浏览器中的计算。
export const MEASURING_STYLE = {
  position: "fixed",
  opacity: "0",
  top: "0",
  left: "0",
} as const;

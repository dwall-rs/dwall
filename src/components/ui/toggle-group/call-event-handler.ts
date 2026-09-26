/**
 * 调用用户传入的事件处理器(兼容 Solid 的函数 / 函数数组 / bound 对象形式)。
 * splitProps 提取出的事件类型是 EventHandlerUnion,不能直接调用。
 */
export function callEventHandler<E extends Event>(
  handler: unknown,
  event: E,
): void {
  const list = Array.isArray(handler) ? handler : [handler];
  for (const h of list) {
    if (typeof h === "function") (h as (e: E) => void)(event);
  }
}

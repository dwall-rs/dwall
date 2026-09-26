/**
 * 极简日志库
 *
 * 两套 API，按场景选用：
 *
 * 1. logger.debug/info/warn/error —— 精确行号版
 *    直接 bind 原生 console 方法，浏览器控制台会指向真实的业务代码调用行，
 *    可点击跳转。生产环境（import.meta.env.DEV === false）自动变成空函数，
 *    构建时会被死代码消除，几乎不占体积、不产生开销。
 *    适合：普通业务日志，调用频率不高。
 *
 * 2. logger.hot(tag) —— 高频热路径版
 *    用于循环内部、高频事件回调等会产生大量日志的场景。
 *    牺牲了精确行号（控制台里显示的是 hot() 内部位置，但会带上 tag 前缀
 *    方便定位），换取：
 *      - 节流：同一个 tag 在 windowMs 毫秒内最多真正输出 maxPerWindow 条，
 *        超出的直接丢弃，窗口结束时输出一条"跳过了 N 条"的汇总，避免刷屏和阻塞主线程。
 *      - 参数惰性求值：可以传函数作为参数，只有真正要输出时才会执行，
 *        被节流丢弃的调用不会白白付出字符串拼接 / JSON.stringify 等开销。
 *    适合：循环体内、高频 WebSocket 消息、频繁触发的事件回调等。
 *
 * 使用：
 *   import { logger } from './logger';
 *
 *   // 普通日志，精确行号
 *   logger.info('用户登录', { userId: 123 });
 *
 *   // 热路径日志，自动节流 + 惰性求值
 *   const logFrame = logger.hot('render-loop', 'debug', { windowMs: 1000, maxPerWindow: 3 });
 *   for (const item of hugeList) {
 *     logFrame(() => `处理 ${item.id}: ${JSON.stringify(item)}`); // 只有真正要输出时才会 JSON.stringify
 *   }
 */

export type LogLevel = "debug" | "info" | "warn" | "error";

export interface LoggerOptions {
  /** 最低输出级别，默认 debug（即全部输出） */
  level?: LogLevel;
  /** 日志前缀，用于区分模块，如 'auth' / 'api' */
  prefix?: string;
}

export interface HotOptions {
  /** 节流时间窗口，毫秒，默认 1000 */
  windowMs?: number;
  /** 每个窗口内最多真正输出多少条，默认 5 */
  maxPerWindow?: number;
}

/** hot() 的参数支持直接值，或者一个惰性求值函数（只有真正要输出时才会调用） */
type HotArg = unknown | (() => unknown);
type HotLogFn = (...args: HotArg[]) => void;
type LogFn = (...args: unknown[]) => void;

const LEVEL_WEIGHT: Record<LogLevel, number> = {
  debug: 0,
  info: 1,
  warn: 2,
  error: 3,
};

const LEVEL_STYLE: Record<LogLevel, string> = {
  debug: "color:#888",
  info: "color:#2b8a3e",
  warn: "color:#e8590c",
  error: "color:#c92a2a;font-weight:bold",
};

// Vite 会在构建期把这一行替换为字面量布尔值
const isDev = import.meta.env.DEV;

const noop: LogFn = () => {};
const noopHot: HotLogFn = () => {};

interface HotState {
  count: number;
  windowStart: number;
  suppressed: number;
}

class Logger {
  /** 精确行号版：构造时就确定好是真正绑定到 console 还是空函数 */
  readonly debug: LogFn;
  readonly info: LogFn;
  readonly warn: LogFn;
  readonly error: LogFn;

  private level: LogLevel;
  private prefix: string;
  private hotState = new Map<string, HotState>();

  constructor(options: LoggerOptions = {}) {
    this.level = options.level ?? "debug";
    this.prefix = options.prefix ?? "";

    this.debug = this.build("debug", console.debug);
    this.info = this.build("info", console.info);
    this.warn = isDev
      ? this.build("warn", console.warn)
      : (console.warn.bind(console, ...this.tagArgs("warn")) as LogFn);
    // error 默认生产环境也保留输出（便于线上排查 / 接入上报）
    // 如需生产环境也完全静默，把下面这行换成 this.build('error', console.error)
    this.error = isDev
      ? this.build("error", console.error)
      : (console.error.bind(console, ...this.tagArgs("error")) as LogFn);
  }

  private tagArgs(level: LogLevel): unknown[] {
    const time = new Date().toLocaleTimeString("zh-CN", { hour12: false });
    const tag = this.prefix ? `[${this.prefix}]` : "";
    return [`%c${time} ${tag} ${level.toUpperCase()}`, LEVEL_STYLE[level]];
  }

  private build(level: LogLevel, method: (...args: unknown[]) => void): LogFn {
    if (!isDev) return noop;
    if (LEVEL_WEIGHT[level] < LEVEL_WEIGHT[this.level]) return noop;
    return method.bind(console, ...this.tagArgs(level)) as LogFn;
  }

  /**
   * 高频热路径日志：节流 + 惰性求值，牺牲精确行号。
   * 每次调用 hot() 会返回一个绑定了 tag 的函数，建议在循环 / 高频回调外部
   * 创建一次，复用同一个函数，而不是每次迭代都调用 logger.hot(...)。
   */
  hot(
    tag: string,
    level: LogLevel = "debug",
    options: HotOptions = {},
  ): HotLogFn {
    if (!isDev) return noopHot;
    if (LEVEL_WEIGHT[level] < LEVEL_WEIGHT[this.level]) return noopHot;

    const windowMs = options.windowMs ?? 1000;
    const maxPerWindow = options.maxPerWindow ?? 5;
    const fullTag = this.prefix ? `${this.prefix}:${tag}` : tag;
    const consoleMethod = level === "debug" ? console.log : console[level];

    return (...args: HotArg[]) => {
      const now = performance.now();
      let state = this.hotState.get(fullTag);

      if (!state || now - state.windowStart > windowMs) {
        if (state && state.suppressed > 0) {
          console.log(
            `%c[${fullTag}] 节流：过去 ${windowMs}ms 内还有 ${state.suppressed} 条日志被跳过`,
            "color:#999;font-style:italic",
          );
        }
        state = { count: 0, windowStart: now, suppressed: 0 };
        this.hotState.set(fullTag, state);
      }

      state.count++;
      if (state.count > maxPerWindow) {
        // 超额直接丢弃，不对参数求值，避免白白付出计算开销
        state.suppressed++;
        return;
      }

      // 惰性求值：只有真正要输出的这几条，才会执行传入的函数参数
      const resolved = args.map((a) =>
        typeof a === "function" ? (a as () => unknown)() : a,
      );
      consoleMethod.call(
        console,
        `%c[${fullTag}]`,
        LEVEL_STYLE[level],
        ...resolved,
      );
    };
  }

  /** 创建带子前缀的子 logger，便于按模块区分 */
  child(prefix: string): Logger {
    return new Logger({
      level: this.level,
      prefix: this.prefix ? `${this.prefix}:${prefix}` : prefix,
    });
  }
}

export { Logger };
export const logger = new Logger();

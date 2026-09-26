# Tabs 组件设计方案(SRP)

> 状态:**已实现**
> 移植目标:**base-ui**(`@base-ui/react` 1.6.0 的 `Tabs` 语义),非 radix
> 关联代码:`src/components/tabs/`

## 1. 背景与现状

- 组件在样式完成的基础上补齐交互,核心语义对齐 base-ui:
  - 激活(active)与焦点高亮(highlighted)**分离**:默认 `activateOnFocus=false` 时,聚焦/键盘导航只移动高亮,不激活;激活只发生在**点击**(鼠标抬起、Enter/Space 触发 button click)。
  - 键盘方向键/Home/End 走 roving focus(仅移动焦点)。
- 项目已确立的组件模式(参照 `accordion`、`collapsible`、`slider`):
  - 状态存于 `Xxx.context.ts(x)` 的 Context + Provider,组件通过 `useXxxContext()` 消费。
  - 受控/非受控双模式:`props.value` 存在即受控(只通知外部),否则内部状态 + 回调。
  - 类型使用 `PolymorphicProps<T, O, false>` + `BaseProps`,样式用 `clsx` + cva。
- 约束:不引入 radix/kobalte/base-ui 等外部运行时依赖,手写实现。

## 2. 设计目标与 SRP 分解

单一职责原则的落点:**一个文件只负责一件事;渲染、状态、交互算法、ARIA 语义互相解耦**。

| 关注点 | 归属文件 | 职责 |
| --- | --- | --- |
| 类型契约 | `Tabs.types.ts` / `TabsTrigger.types.ts` / `TabsContent.types.ts` / `TabsList.types.ts` | 只声明 props 类型,不含实现 |
| 根状态 | `Tabs/Tabs.context.tsx` | 激活值(受控/非受控)、高亮值、注册表、id 关联 + Provider + `useTabsContext` |
| 列表状态 | `TabsList/TabsList.context.ts` | `activateOnFocus` 传递 + `useTabsListContext` |
| 交互算法 | `Tabs/useTabsKeyboard.ts` | 纯函数式键盘导航(方向键/Home/End/loop/rtl/disabled 跳过),可独立单测 |
| 事件组合 | `Tabs/call-event-handler.ts` | 调用用户事件处理器(兼容函数/数组),组件内事件与用户事件不互相覆盖 |
| 根渲染 | `Tabs/Tabs.tsx` | 组装 Provider、输出根 DOM、`data-orientation`/`dir`/`data-vertical` 等 |
| 列表语义 | `TabsList/TabsList.tsx` | `role="tablist"`、`aria-orientation`、挂载键盘事件、variant 样式、提供 ListContext |
| 触发器语义 | `TabsTrigger/TabsTrigger.tsx` | `role="tab"`、激活态、高亮态、roving tabindex、点击激活、ARIA 关联 |
| 面板语义 | `TabsContent/TabsContent.tsx` | `role="tabpanel"`、显隐控制、惰性渲染 |
| 样式 | `TabsList.styles.ts`、`TabsTrigger.styles.ts` | class 字符串与 tsx 分离,与组件逻辑解耦 |

### 文件结构

```
src/components/tabs/
├── index.ts
├── Tabs/
│   ├── index.ts
│   ├── Tabs.tsx               # 根容器 + Provider 组装(不含状态逻辑)
│   ├── Tabs.types.ts          # 类型契约
│   ├── Tabs.context.tsx       # 唯一状态源(受控/非受控 + 高亮 + 注册表 + id 关联)
│   ├── useTabsKeyboard.ts     # 键盘导航纯逻辑
│   └── call-event-handler.ts  # 事件处理器调用工具(函数/数组)
├── TabsList/
│   ├── index.ts
│   ├── TabsList.tsx           # tablist 容器 + 键盘事件绑定 + ListContext Provider
│   ├── TabsList.context.ts    # activateOnFocus 传递
│   ├── TabsList.styles.ts
│   └── TabsList.types.ts
├── TabsTrigger/
│   ├── index.ts
│   ├── TabsTrigger.tsx        # tab 语义 + 激活/高亮 + roving tabindex
│   ├── TabsTrigger.styles.ts
│   └── TabsTrigger.types.ts
└── TabsContent/
    ├── index.ts
    ├── TabsContent.tsx        # tabpanel 语义 + 显隐
    └── TabsContent.types.ts
```

## 3. 类型契约

```ts
// Tabs/Tabs.types.ts
export type TabsValue = string | number;

interface BaseTabsProps extends BaseProps {
  defaultValue?: TabsValue;              // 非受控初始激活值
  value?: TabsValue;                     // 受控激活值
  onValueChange?: (value: TabsValue) => void;
  orientation?: "horizontal" | "vertical"; // 默认 "horizontal"
  loop?: boolean;                        // 方向键循环,默认 true
}

// TabsList/TabsList.types.ts(新增)
type BaseTabsListProps = BaseProps & VariantProps<typeof tabsListVariants> & {
  activateOnFocus?: boolean;             // 聚焦是否激活,默认 false(base-ui 语义)
};

// TabsTrigger/TabsTrigger.types.ts
type BaseTabsTriggerProps = BaseProps & {
  value: TabsValue;
  onClick?: MouseEventHandler<"button">; // 可选,组件内部会先处理激活再调用用户回调
};
// disabled 复用 button 原生属性

// TabsContent/TabsContent.types.ts
type BaseTabsContentProps = BaseProps & {
  value: TabsValue;
  forceMount?: boolean;                  // true 时始终挂载 DOM(仅隐藏)
};
```

## 4. 状态管理(`Tabs.context.tsx`)

单选组件,用 `createSignal`;两个独立状态:

```ts
interface TabsContextValue {
  value: Accessor<TabsValue | undefined>;        // 激活值(受控优先)
  setValue: (v: TabsValue) => void;
  isSelected: (v: TabsValue) => boolean;
  orientation / dir / loop: Accessor<...>;
  highlightedValue: Accessor<TabsValue | undefined>;  // 焦点高亮,与激活分离
  setHighlightedValue: (v: TabsValue) => void;
  registerTrigger: (entry: { value; disabled: () => boolean; element }) => () => void;
  getTriggers: () => TriggerEntry[];             // 键盘导航数据源
  isFirstEnabled: (v) => boolean;                // 首个可用 trigger(tabindex 起点)
  setTriggerId / getTriggerId / setContentId / getContentId;  // aria 交叉引用
}
```

- **受控/非受控**:`props.value` 存在即受控(只通知外部),否则内部 signal + 回调;`props` 经 `mergeProps` 为响应式 getter。
- **注册表**:`TriggerEntry.disabled` 存 accessor(支持运行时翻转),`element` 由 trigger 的 ref 提供;`onCleanup` 注销,支持动态增删。
- **id 关联**:`createUniqueId` + 按 `value` 键注册 triggerId/contentId,响应式交叉读取。
- 嵌套场景:Context 就近取用,天然支持嵌套 tabs。

## 5. 激活与焦点模型(base-ui 语义)

| 交互 | 行为 |
| --- | --- |
| 鼠标点击 | `onClick` 激活(鼠标抬起后;mousedown 触发的 focus 不激活) |
| Enter / Space | button 原生 click → 激活 |
| 键盘方向键 / Home / End | 只移动焦点(更新高亮),**不激活** |
| 聚焦(`activateOnFocus=false`,默认) | 只更新高亮 |
| 聚焦(`activateOnFocus=true`) | 聚焦即激活(由 `TabsTrigger.onFocus` 处理) |

实现要点:

- `TabsTrigger.onClick`:非 disabled 时 `ctx.setValue(value)`,再调用户回调。
- `TabsTrigger.onFocus`:非 disabled 时 `setHighlightedValue(value)`;`activateOnFocus` 时再 `setValue(value)`。
- `TabsList` 的 `onKeyDown` 走 `useTabsKeyboard`:计算下一 tab 并 `.focus()`(触发 onFocus 更新高亮),不直接激活。

## 6. 键盘导航(`useTabsKeyboard.ts`)

```
触发条件:e.currentTarget 内有 role="tab" 且焦点在某个非禁用 tab 上
方向映射(按 orientation + dir):
  horizontal + ltr: ArrowRight → next, ArrowLeft → prev
  horizontal + rtl: 反向(dir="auto" 时回退文档实际方向)
  vertical:          ArrowDown → next, ArrowUp → prev
  Home → 第一个,End → 最后一个
loop=false 边界停止,loop=true 环绕;跳过 disabled
```

数据源为 context 注册表(`getTriggers`),value 类型保真(不依赖 DOM `data-*` 字符串化)。

**roving tabindex**(`TabsTrigger` 内计算):

```ts
tabIndex = disabled ? -1
         : highlighted === value && isUsable(value) ? 0                    // 焦点所在且可用
         : (highlighted === undefined || !isUsable(highlighted))           // 高亮缺失/不可用
           && isFirstEnabled(value) ? 0                                    // 回退首个可用起点
         : -1;
```

- 高亮优先;高亮为空或指向 disabled/已卸载 trigger 时,首个可用 tab 持 0,避免 tablist 键盘不可达。
- 激活值变化时,若焦点不在 tab 上,高亮自动同步到激活 tab(对齐 base-ui;键盘 roving 过程中不覆盖)。

## 7. 无障碍(ARIA)

| 元素 | 属性 | 值 |
| --- | --- | --- |
| `TabsList` | `role` / `aria-orientation` | `"tablist"` / vertical 时 `"vertical"` |
| `TabsTrigger` | `role` | `"tab"` |
| `TabsTrigger` | `id` | 生成的 `triggerId` |
| `TabsTrigger` | `aria-selected` / `aria-controls` / `aria-disabled` | `isSelected` / contentId / disabled |
| `TabsTrigger` | `tabindex` | 见 roving tabindex |
| `TabsTrigger` | `disabled` | 原生 disabled(阻止点击/聚焦) |
| `TabsContent` | `role` / `id` / `aria-labelledby` / `tabindex` | `"tabpanel"` / contentId / triggerId / `0` |

激活元素输出 `data-active`,与现有样式(`data-active:bg-background` 等)接通。

## 8. 渲染与样式衔接

- `Tabs` 根:`data-orientation` 之外,同时渲染 `data-vertical` / `data-horizontal`,使现有 `group-data-vertical/tabs:`、`group-data-horizontal/tabs:` 选择器生效(实现时发现并补齐的不一致)。
- `TabsContent`:默认 `<Show>` 惰性渲染(未激活不挂载);`forceMount` 时挂载但隐藏——因 `flex-1`(display:flex)会覆盖 UA 的 `[hidden]`,隐藏必须显式 `display:none`(内联 style,合并用户 style)。
- 事件处理:组件内部事件与用户事件通过 `callEventHandler` 组合,互不覆盖(splitProps restKeys 提取用户处理器)。

## 9. 边界情况与决策记录

1. **无 `defaultValue` 且非受控**:无激活 tab,首个可用 trigger 持 `tabindex=0` 作为键盘起点。
2. **激活值指向 disabled/已删除 trigger**:内容随 `isSelected` 消失;tabindex 起点回落到首个可用 tab。
3. **disabled trigger**:原生 `disabled`(不可点击/聚焦)+ `aria-disabled` + 键盘导航跳过 + 注册表 accessor 实时读取。
4. **RTL**:`dir="rtl"` 反转左右键;`dir="auto"` 回退文档实际方向。
5. **受控判定**:`props.value !== undefined`;若未来需"显式清空激活"语义,改 `"value" in props` 并允许 `null`(决策点)。
6. **运行时 disabled 翻转**:注册表存 accessor,`isFirstEnabled` 与键盘跳过实时生效。
7. **多实例/嵌套**:状态全在 Provider 内;嵌套 tabs 由 Context 就近取用。
8. **性能**:`createSignal` 足够;大规模场景可迁移 `createStore` + selector,保持 context 接口不变。

## 10. 测试计划

项目当前无测试设施,建议最小引入 `vitest` + `solid-testing-library` + `jsdom`,覆盖:

- **激活时机**:鼠标 click 激活、mousedown 不激活;Enter/Space 激活;`activateOnFocus` 开关。
- **键盘导航**:方向映射(horizontal/vertical、ltr/rtl/auto)、Home/End、loop、跳过 disabled;方向键不激活(默认)。
- **状态**:非受控点击 + `onValueChange`;受控只回调不改内部;`defaultValue` 初始激活。
- **渲染**:激活 content 显示、非激活不挂载;`forceMount` 挂载但隐藏且 `display:none`。
- **ARIA**:`aria-selected`/`aria-controls`/`aria-labelledby`/tabindex 起点。
- **动态增删**:删除激活 tab 后 tabindex 起点回落;运行时 disabled 翻转。

## 11. 实施记录

已完成:类型契约 → context(状态/注册表/id 关联)→ 键盘 hook → TabsList(role/键盘/ListContext)→ TabsTrigger(激活/高亮/tabindex)→ TabsContent(惰性/forceMount)→ Tabs 根(Provider/data-*)→ base-ui 语义修正(激活时机、高亮分离)→ 样式抽取与样式钩子补齐。验证:`tsc --noEmit` + `biome check` 通过。

## 附:使用示例(预期 API)

```tsx
<Tabs defaultValue="account">
  <TabsList>
    <TabsTrigger value="account">账号</TabsTrigger>
    <TabsTrigger value="password">密码</TabsTrigger>
  </TabsList>
  <TabsContent value="account">账号设置</TabsContent>
  <TabsContent value="password">密码设置</TabsContent>
</Tabs>
```

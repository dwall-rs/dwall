import { createSignal, For, Show, type Component } from "solid-js";
import styles from "./index.module.scss";
import type { SelectProps, SelectOption } from "./types";
import InputContainer from "./InputContainer";
import { createEffect, onCleanup } from "solid-js";

const Select: Component<SelectProps> = (props) => {
  const [isOpen, setIsOpen] = createSignal(false);
  const [focused, setFocused] = createSignal(false);
  const [selectedValue, setSelectedValue] = createSignal<string | undefined>(
    props.value,
  );
  const [warning, setWarning] = createSignal<string | undefined>();

  // 引用下拉菜单和选择器容器
  let dropdownRef: HTMLDivElement | undefined;
  let selectRef: HTMLDivElement | undefined;

  // 当props.value变化时更新内部状态
  createEffect(() => {
    setSelectedValue(props.value);
  });

  // 处理点击外部关闭下拉菜单
  const handleClickOutside = (event: MouseEvent) => {
    if (selectRef && !selectRef.contains(event.target as Node) && isOpen()) {
      setIsOpen(false);
    }
  };

  // 添加和清理点击事件监听器
  createEffect(() => {
    if (isOpen()) {
      document.addEventListener("mousedown", handleClickOutside);
    } else {
      document.removeEventListener("mousedown", handleClickOutside);
    }

    onCleanup(() => {
      document.removeEventListener("mousedown", handleClickOutside);
    });
  });

  // 切换下拉菜单的开关状态
  const toggleDropdown = () => {
    if (!props.disabled) {
      setIsOpen(!isOpen());
    }
  };

  // 处理选项选择
  const handleOptionSelect = (option: SelectOption) => {
    if (option.disabled) return;

    setSelectedValue(option.value);
    setIsOpen(false);
    props.onChange?.(option.value);
  };

  // 获取当前选中选项的标签
  const getSelectedLabel = () => {
    const value = selectedValue();
    if (value === undefined) return undefined;

    const option = props.options.find((opt) => opt.value === value);
    return option?.label;
  };

  // 计算选择器容器的类名
  const selectWrapperClass = () => {
    const classes = [styles.selectWrapper];
    if (focused()) classes.push(styles.focused);
    if (props.disabled) classes.push(styles.disabled);
    return classes.join(" ");
  };

  // 计算下拉菜单的类名
  const dropdownClass = () => {
    const classes = [styles.dropdown];
    if (isOpen()) classes.push(styles.open);
    return classes.join(" ");
  };

  // 计算箭头图标的类名
  const arrowClass = () => {
    const classes = [styles.arrow];
    if (isOpen()) classes.push(styles.open);
    return classes.join(" ");
  };

  // 计算选项的类名
  const optionClass = (option: SelectOption) => {
    const classes = [styles.option];
    if (option.disabled) classes.push(styles.disabled);
    return classes.join(" ");
  };

  return (
    <InputContainer
      label={props.label}
      required={props.required}
      warning={warning()}
    >
      <div
        ref={selectRef}
        class={selectWrapperClass()}
        onClick={toggleDropdown}
        onFocus={() => setFocused(true)}
        onBlur={() => setFocused(false)}
        style={props.style}
        tabIndex={props.disabled ? undefined : 0}
      >
        <Show
          when={getSelectedLabel()}
          fallback={
            <div class={styles.placeholder}>
              {props.placeholder || "请选择"}
            </div>
          }
        >
          <div class={styles.value}>{getSelectedLabel()}</div>
        </Show>
        <div class={arrowClass()}>▼</div>

        <div ref={dropdownRef} class={dropdownClass()}>
          <For each={props.options}>
            {(option) => (
              <div
                class={optionClass(option)}
                onClick={(e) => {
                  e.stopPropagation();
                  handleOptionSelect(option);
                }}
              >
                {option.label}
              </div>
            )}
          </For>
        </div>
      </div>
    </InputContainer>
  );
};

export default Select;

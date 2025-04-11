import {
  createEffect,
  onCleanup,
  createUniqueId,
  createSignal,
  For,
  Show,
  type Component,
} from "solid-js";
import { BiSolidChevronDown } from "solid-icons/bi";
import type { SelectProps, SelectOption } from "./types";
import InputContainer from "./InputContainer";
import styles from "./index.module.scss";

const Select: Component<SelectProps> = (props) => {
  const [isOpen, setIsOpen] = createSignal(false);
  const [focused, setFocused] = createSignal(false);
  const [selectedValue, setSelectedValue] = createSignal<string | undefined>(
    props.value,
  );

  // 创建唯一ID用于ARIA属性
  const selectId = createUniqueId();
  const listboxId = `${selectId}-listbox`;

  // 引用下拉菜单和选择器容器
  let dropdownRef: HTMLDivElement | undefined;
  let selectRef: HTMLDivElement | undefined;
  const optionsRef: HTMLOptionElement[] = [];

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
      const newIsOpen = !isOpen();
      setIsOpen(newIsOpen);

      if (newIsOpen) {
        // 当打开下拉菜单时，高亮当前选中项
        const selectedIndex = props.options.findIndex(
          (opt) => opt.value === selectedValue(),
        );
        // 下一帧滚动到选中项
        setTimeout(() => {
          if (optionsRef[selectedIndex]) {
            optionsRef[selectedIndex].scrollIntoView({ block: "nearest" });
          }
        }, 0);
      }
    }
  };

  // 处理选项选择
  const handleOptionSelect = (option: SelectOption) => {
    if (option.disabled) return;

    setSelectedValue(option.value);
    setIsOpen(false);
    props.onChange?.(option.value);
  };

  // // 处理键盘导航
  // const handleKeyDown = (event: KeyboardEvent) => {
  //   if (props.disabled) return;

  //   switch (event.key) {
  //     case "ArrowDown":
  //       event.preventDefault();
  //       break;

  //     case "ArrowUp":
  //       event.preventDefault();
  //       break;

  //     case "Enter":
  //     case " ":
  //       event.preventDefault();
  //       break;

  //     case "Escape":
  //       event.preventDefault();
  //       if (isOpen()) {
  //         setIsOpen(false);
  //       }
  //       break;

  //     case "Tab":
  //       if (isOpen()) {
  //         setIsOpen(false);
  //       }
  //       break;

  //     case "Home":
  //       if (isOpen()) {
  //         event.preventDefault();
  //       }
  //       break;

  //     case "End":
  //       if (isOpen()) {
  //         event.preventDefault();
  //       }
  //       break;
  //   }
  // };

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
    if (option.value === selectedValue()) classes.push(styles.selected);
    return classes.join(" ");
  };

  return (
    <InputContainer
      label={props.label}
      required={props.required}
      labelId={`${selectId}-label`}
    >
      <div
        ref={selectRef}
        class={selectWrapperClass()}
        onClick={toggleDropdown}
        onFocus={() => setFocused(true)}
        onBlur={() => setFocused(false)}
        // onKeyDown={handleKeyDown}
        style={props.style}
        tabIndex={props.disabled ? undefined : 0}
        // biome-ignore lint/a11y/useSemanticElements: <explanation>
        role="combobox"
        aria-haspopup="listbox"
        aria-expanded={isOpen()}
        aria-disabled={props.disabled}
        aria-controls={listboxId}
        aria-labelledby={props.label ? `${selectId}-label` : undefined}
        id={selectId}
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
        <div class={arrowClass()}>
          <BiSolidChevronDown size={16} />
        </div>

        <div
          tabIndex={0}
          ref={dropdownRef}
          class={dropdownClass()}
          // biome-ignore lint/a11y/useSemanticElements: <explanation>
          role="listbox"
          id={listboxId}
          aria-labelledby={props.label ? `${selectId}-label` : undefined}
        >
          <For each={props.options}>
            {(option, index) => (
              <option
                tabIndex={0}
                ref={(el) => {
                  optionsRef[index()] = el;
                }}
                class={optionClass(option)}
                onClick={(e) => {
                  e.stopPropagation();
                  handleOptionSelect(option);
                }}
                aria-selected={option.value === selectedValue()}
                aria-disabled={option.disabled}
              >
                {option.label}
              </option>
            )}
          </For>
        </div>
      </div>
    </InputContainer>
  );
};

export default Select;

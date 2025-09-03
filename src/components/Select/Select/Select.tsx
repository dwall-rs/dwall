import {
  createEffect,
  onCleanup,
  createUniqueId,
  createSignal,
  For,
  Show,
  type Component,
} from "solid-js";
import { VsChevronDown } from "solid-icons/vs";
import type { SelectProps, SelectOption } from "./Select.types";
import SelectContainer from "../SelectContainer";
import styles from "./Select.css";

const Select: Component<SelectProps> = (props) => {
  const [isOpen, setIsOpen] = createSignal(false);
  const [focused, setFocused] = createSignal(false);
  const [selectedValue, setSelectedValue] = createSignal<string | undefined>(
    props.value,
  );

  // Create unique ID for ARIA attributes
  const selectId = createUniqueId();
  const listboxId = `${selectId}-listbox`;

  // References to dropdown menu and selector container
  let dropdownRef: HTMLDivElement | undefined;
  let selectRef: HTMLDivElement | undefined;
  const optionsRef: HTMLOptionElement[] = [];

  // Update internal state when props.value changes
  createEffect(() => {
    setSelectedValue(props.value);
  });

  // Handle clicking outside to close dropdown menu
  const handleClickOutside = (event: MouseEvent) => {
    if (selectRef && !selectRef.contains(event.target as Node) && isOpen()) {
      setIsOpen(false);
    }
  };

  // Add and clean up click event listener
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

  // Toggle dropdown menu open/close state
  const toggleDropdown = () => {
    if (!props.disabled) {
      const newIsOpen = !isOpen();
      setIsOpen(newIsOpen);

      if (newIsOpen) {
        // When opening dropdown menu, highlight current selected item
        const selectedIndex = props.options.findIndex(
          (opt) => opt.value === selectedValue(),
        );
        // Scroll to selected item on next frame
        setTimeout(() => {
          if (optionsRef[selectedIndex]) {
            optionsRef[selectedIndex].scrollIntoView({ block: "nearest" });
          }
        }, 0);
      }
    }
  };

  // Handle option selection
  const handleOptionSelect = (option: SelectOption) => {
    if (option.disabled) return;

    setSelectedValue(option.value);
    setIsOpen(false);
    props.onChange?.(option.value);
  };

  // Get label of current selected option
  const getSelectedLabel = () => {
    const value = selectedValue();
    if (value === undefined) return undefined;

    const option = props.options.find((opt) => opt.value === value);
    return option?.label;
  };

  // Calculate selector container class name
  const selectWrapperClass = () => {
    const classes = [styles.selectWrapper];
    if (focused()) classes.push(styles.selectWrapperFocused);
    if (props.disabled) classes.push(styles.selectWrapperDisabled);
    return classes.join(" ");
  };

  // Calculate dropdown menu class name
  const dropdownClass = () => {
    const classes = [styles.dropdown];
    if (isOpen()) classes.push(styles.dropdownOpen);
    return classes.join(" ");
  };

  // Calculate arrow icon class name
  const arrowClass = () => {
    const classes = [styles.arrow];
    if (isOpen()) classes.push(styles.arrowOpen);
    return classes.join(" ");
  };

  // Calculate option class name
  const optionClass = (option: SelectOption) => {
    const classes = [styles.option];
    if (option.disabled) classes.push(styles.optionDisabled);
    if (option.value === selectedValue()) classes.push(styles.optionSelected);
    return classes.join(" ");
  };

  return (
    <SelectContainer
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
        style={props.style}
        tabIndex={props.disabled ? undefined : 0}
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
              {props.placeholder || "Select..."}
            </div>
          }
        >
          <div class={styles.value}>{getSelectedLabel()}</div>
        </Show>
        <div class={arrowClass()}>
          <VsChevronDown size={16} />
        </div>

        <div
          tabIndex={0}
          ref={dropdownRef}
          class={dropdownClass()}
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
    </SelectContainer>
  );
};

export default Select;

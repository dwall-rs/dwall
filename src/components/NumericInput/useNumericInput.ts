import { createSignal } from "solid-js";
import type { NumericInputProps } from "./types";
import { useAppContext } from "~/context";
import { translate } from "~/utils/i18n";

export const numberValidation = {
  isValidNumberInput: (value: string): boolean => {
    if (value === "" || value === "-" || value === ".") return true;
    return (
      /^-?\d*\.?\d*$/.test(value) && !Number.isNaN(Number.parseFloat(value))
    );
  },

  validateRange: (
    value: number,
    min?: { value?: number; warning: string },
    max?: { value?: number; warning: string },
  ): {
    isValid: boolean;
    message: string;
    value: number;
  } => {
    if (min?.value !== undefined && value < min.value) {
      return { isValid: false, message: min.warning, value: min.value };
    }
    if (max?.value !== undefined && value > max.value) {
      return { isValid: false, message: max.warning, value: max.value };
    }
    return { isValid: true, message: "", value };
  },
};

const useNumericInputHandling = (props: NumericInputProps) => {
  const { translations } = useAppContext();
  const [inputValue, setInputValue] = createSignal(
    props.value?.toString() || "",
  );
  const [warning, setWarning] = createSignal<string>("");

  const invalidNumberMessage = translate(
    translations()!,
    "message-invalid-number-input",
  );
  const tooSmallMessage = props.min
    ? translate(translations()!, "message-number-too-small", {
        min: String(props.min),
      })
    : "";
  const tooLargeMessage = props.max
    ? translate(translations()!, "message-number-too-large", {
        max: String(props.max),
      })
    : "";

  const handleBlur = () => {
    const value = inputValue();
    if (value === "" || value === "-" || value === ".") {
      setInputValue("");
      props.onChange?.();
      return;
    }

    if (!numberValidation.isValidNumberInput(value)) {
      setWarning(invalidNumberMessage);
      return;
    }

    const numValue = Number.parseFloat(value);
    const { message, value: validatedValue } = numberValidation.validateRange(
      numValue,
      {
        value: props.min,
        warning: tooSmallMessage,
      },
      {
        value: props.max,
        warning: tooLargeMessage,
      },
    );
    setWarning(message);
    setInputValue(validatedValue.toString());
    props.onChange?.(validatedValue);
  };

  const handleInput = (value: string) => {
    if (value === "" || value === "-" || value === ".") {
      setInputValue(value);
      setWarning("");
      props.onChange?.();
      return;
    }

    if (!numberValidation.isValidNumberInput(value)) {
      setWarning(invalidNumberMessage);
      props.onChange?.();
      return;
    }

    const numValue = Number.parseFloat(value);
    const { message } = numberValidation.validateRange(
      numValue,
      {
        value: props.min,
        warning: tooSmallMessage,
      },
      {
        value: props.max,
        warning: tooLargeMessage,
      },
    );
    setWarning(message);
    setInputValue(value);
    props.onChange?.(numValue);
  };

  return {
    inputValue,
    setInputValue,
    warning,
    setWarning,
    handleBlur,
    handleInput,
    tooSmallMessage,
    tooLargeMessage,
  };
};

export default useNumericInputHandling;

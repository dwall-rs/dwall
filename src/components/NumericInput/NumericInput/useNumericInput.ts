import { createSignal } from "solid-js";
import type { NumericInputProps } from "./NumericInput.types";

import { useTranslations } from "~/contexts";

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
  const { translate } = useTranslations();
  const [inputValue, setInputValue] = createSignal(
    props.value?.toString() || "",
  );
  const [warning, setWarning] = createSignal<string>("");

  const invalidNumberMessage = translate("message-invalid-number-input");
  const tooSmallMessage = props.min
    ? translate("message-number-too-small", {
        min: String(props.min),
      })
    : "";
  const tooLargeMessage = props.max
    ? translate("message-number-too-large", {
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
    if (message) {
      setWarning(message);
      props.onChange?.();
    } else {
      setWarning("");
      props.onChange?.(validatedValue);
    }
    setInputValue(validatedValue.toString());
  };

  const handleCommonLogic = (
    value: string,
    callback?: (value?: number) => void,
  ) => {
    if (value === "" || value === "-" || value === ".") {
      setInputValue(value);
      setWarning("");
      callback?.();
      return;
    }

    if (!numberValidation.isValidNumberInput(value)) {
      setWarning(invalidNumberMessage);
      callback?.();
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

    if (message) {
      setWarning(message);
      callback?.();
    } else {
      setWarning("");
      callback?.(numValue);
    }
    setInputValue(value);
  };

  const handleInput = (value: string) => {
    handleCommonLogic(value, props.onInput);
  };

  const handleChange = (value: string) => {
    handleCommonLogic(value, props.onChange);
  };
  return {
    inputValue,
    setInputValue,
    warning,
    setWarning,
    handleBlur,
    handleInput,
    handleChange,
    tooSmallMessage,
    tooLargeMessage,
  };
};

export default useNumericInputHandling;

import { createContext, useContext, type Setter } from "solid-js";

interface AlertDialogContentContextValue {
  setTitleID: Setter<string | undefined>;
  setDescriptionID: Setter<string | undefined>;
}

export const AlertDialogContentContext =
  createContext<AlertDialogContentContextValue>();

export const useAlertDialogContentContext = () => {
  const ctx = useContext(AlertDialogContentContext);
  if (!ctx)
    throw new Error(
      "useAlertDialogContentContext must be used within an AlertDialogContentProvider",
    );

  return ctx;
};

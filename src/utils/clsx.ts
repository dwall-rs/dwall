import { twMerge } from "tailwind-merge";

export const clsx = (...classes: (string | undefined | false)[]): string => {
  return twMerge(classes.filter(Boolean));
};

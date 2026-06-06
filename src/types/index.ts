export * from "./props.types";
export * from "./theme.types";

export type MakeRequired<T, K extends keyof T> = Omit<T, K> &
  Required<Pick<T, K>>;

export type TimeoutID = ReturnType<typeof setTimeout>;

type Update = import("@tauri-apps/plugin-updater").Update;

type Accessor<T> = import("solid-js").Accessor<T>;
type Resource<T> = import("solid-js").Resource<T>;
type Setter<T> = import("solid-js").Setter<T>;
type Refetcher<T> = () => T | Promise<T | null | undefined> | null | undefined;

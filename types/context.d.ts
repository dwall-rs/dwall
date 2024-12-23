type Update = import("@tauri-apps/plugin-updater").Update;

type Accessor<T> = import("solid-js").Accessor<T>;
type Resource<T> = import("solid-js").Resource<T>;
type Setter<T> = import("solid-js").Setter<T>;

interface AppContext {
  update: {
    resource: Resource<Update | null>;
    refetch: () => void;
    showDialog: Accessor<boolean | undefined>;
    setShowDialog: Setter<boolean | undefined>;
  };
  config: Accessor<Config | undefined>;
  refetchConfig: () => void;
  settings: { show: Accessor<boolean>; setShow: Setter<boolean> };
}

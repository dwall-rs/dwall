type Accessor<T> = import("solid-js").Accessor<T>;
type Resource<T> = import("solid-js").Resource<T>;
type Setter<T> = import("solid-js").Setter<T>;

interface AppContext {
  config: Accessor<Config | undefined>;
  settings: { show: Accessor<boolean>; setShow: Setter<boolean> };
}

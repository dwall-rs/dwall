type Update = import("@tauri-apps/plugin-updater").Update;

type Accessor<T> = import("solid-js").Accessor<T>;
type Resource<T> = import("solid-js").Resource<T>;
type Setter<T> = import("solid-js").Setter<T>;

interface AppContext {
  // 更新相关
  update: {
    resource: Resource<Update | null>;
    refetch: () => void;
    showDialog: Accessor<boolean | undefined>;
    setShowDialog: Setter<boolean | undefined>;
  };

  // 配置相关
  config: Accessor<Config | undefined>;
  refetchConfig: () => void;

  // 设置相关
  settings: { show: Accessor<boolean>; setShow: Setter<boolean> };

  // 主题相关
  theme: {
    currentTheme: Accessor<ThemeItem | undefined>;
    appliedThemeID: Accessor<string | undefined>;
    setAppliedThemeID: Setter<string | undefined>;
    downloadThemeID: Accessor<string | undefined>;
    setDownloadThemeID: Setter<string | undefined>;
    menuItemIndex: Accessor<number | undefined>;
    setMenuItemIndex: Setter<number | undefined>;
    themeExists: Accessor<boolean>;
    handleThemeSelection: (index: number) => void;
    handleThemeApplication: () => Promise<void>;
  };

  // 监视器相关
  monitor: {
    list: Accessor<Monitor[]>;
    handleChange: (id: string) => void;
    id: Accessor<string | undefined>;
  };

  // 任务相关
  task: {
    handleClosure: () => void;
  };
}

import { AiOutlineCheck, AiOutlineClose } from "solid-icons/ai";
import { createEffect, createSignal, onMount, useContext } from "solid-js";
import { checkAutoStart, disableAutoStart, enableAutoStart } from "~/commands";
import { AppContext } from "~/context";
import {
  LazyButton,
  LazyDialog,
  LazyFlex,
  LazyInputNumber,
  LazyLabel,
  LazySpace,
  LazySwitch,
} from "~/lazy";

const Settings = () => {
  const {
    config: defaultConfig,
    settings: { setShow, show },
  } = useContext(AppContext)!;

  const [config, setConfig] = createSignal(defaultConfig());
  const [autoStartState, setAutoStartState] = createSignal(false);

  createEffect(() => setConfig(defaultConfig()));

  onMount(async () => {
    const state = await checkAutoStart();
    setAutoStartState(state);
  });

  const onSwitchAutoStart = async () => {
    if (autoStartState()) {
      await disableAutoStart();
    } else {
      await enableAutoStart();
    }
    setAutoStartState((prev) => !prev);
  };

  const close = () => setShow(false);

  const onOk = async () => {
    close();
  };

  const onCancel = () => {
    close();
  };

  return (
    <LazyDialog show={show()} onClose={() => { }} maskClosable={false}>
      <LazyFlex direction="vertical" gap={16} style={{ "min-width": "400px" }}>
        <LazyFlex justify="round">
          <LazySpace>
            <LazyLabel>开机自启</LazyLabel>
            <LazySwitch
              checked={autoStartState()}
              setChecked={onSwitchAutoStart}
              checkedChild={<AiOutlineCheck />}
              uncheckedChild={<AiOutlineClose />}
            />
          </LazySpace>

          <LazySpace>
            <LazyLabel>检测间隔</LazyLabel>
            <LazySpace gap={8}>
              <LazyInputNumber value={config()?.interval} />秒
            </LazySpace>
          </LazySpace>
        </LazyFlex>

        <LazySpace justify="around">
          <LazyButton danger onClick={onCancel}>
            取消
          </LazyButton>
          <LazyButton onClick={onOk}>确认</LazyButton>
        </LazySpace>
      </LazyFlex>
    </LazyDialog>
  );
};

export default Settings;

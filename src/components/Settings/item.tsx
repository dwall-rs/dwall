import { children, type JSXElement } from "solid-js";
import { LazyFlex, LazyLabel } from "~/lazy";

interface SettingsItemProps {
  label: string;
  children: JSXElement;
  vertical?: boolean;
}

const SettingsItem = (props: SettingsItemProps) => {
  const resolved = children(() =>
    props.vertical ? (
      <LazyFlex direction="vertical" gap={8}>
        <LazyLabel weight="semibold">{props.label}</LazyLabel>

        {props.children}
      </LazyFlex>
    ) : (
      <LazyFlex justify="between">
        <LazyLabel
          weight="semibold"
          style={{
            display: "flex",
            "justify-items": "center",
            "align-items": "center",
          }}
        >
          {props.label}
        </LazyLabel>

        {props.children}
      </LazyFlex>
    ),
  );

  return <>{resolved}</>;
};

export default SettingsItem;

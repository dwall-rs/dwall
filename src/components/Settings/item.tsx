import { children, type JSXElement } from "solid-js";
import { LazyCol, LazyFlex, LazyLabel, LazyRow } from "~/lazy";

interface SettingsItemProps {
  label: string;
  children: JSXElement;
  vertical?: boolean;
}

const SettingsItem = (props: SettingsItemProps) => {
  const resolved = children(() =>
    props.vertical ? (
      <LazyFlex direction="vertical" gap={8}>
        <LazyLabel>{props.label}</LazyLabel>

        {props.children}
      </LazyFlex>
    ) : (
      <LazyRow>
        <LazyCol span={8}>
          <LazyLabel>{props.label}</LazyLabel>
        </LazyCol>

        <LazyCol span={16}>{props.children}</LazyCol>
      </LazyRow>
    ),
  );

  return <>{resolved}</>;
};

export default SettingsItem;

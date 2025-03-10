import { children, type JSXElement } from "solid-js";
import { LazyFlex, LazyLabel } from "~/lazy";

interface BaseProps {
  label: string;
  children: JSXElement;
}

interface VerticalLayout {
  layout: "vertical";
  vertical: true;
  extra?: never;
}

interface HorizontalLayout {
  layout: "horizontal";
  vertical?: never;
  extra?: JSXElement;
}

interface DefaultLayout {
  layout?: never;
  vertical?: never;
  extra?: never;
}

type LayoutConfig = VerticalLayout | HorizontalLayout | DefaultLayout;
type SettingsItemProps = BaseProps & LayoutConfig;

const labelStyles = {
  display: "flex",
  "justify-items": "center",
  "align-items": "center",
} as const;

const SettingsItem = (props: SettingsItemProps) => {
  const renderLabel = children(() => (
    <LazyLabel weight="semibold" style={labelStyles}>
      {props.label}
    </LazyLabel>
  ));

  const renderContent = children(() => {
    if (props.layout === "vertical") {
      return (
        <LazyFlex direction="vertical" gap={8}>
          {renderLabel()}
          {props.children}
        </LazyFlex>
      );
    }

    const mainContent = (
      <LazyFlex justify="between">
        {renderLabel()}
        {props.children}
      </LazyFlex>
    );

    if (props.layout === "horizontal" && props.extra) {
      return (
        <LazyFlex direction="vertical" gap={8}>
          {mainContent}
          {props.extra}
        </LazyFlex>
      );
    }

    return mainContent;
  });

  return <>{renderContent()}</>;
};

export default SettingsItem;

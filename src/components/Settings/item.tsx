import { children, type JSXElement } from "solid-js";
import {
  LazyButton,
  LazyFlex,
  LazyLabel,
  LazySpace,
  LazyTooltip,
} from "~/lazy";
import { AiOutlineInfoCircle } from "solid-icons/ai";

interface BaseProps {
  label: string;
  children: JSXElement;
  help?: JSXElement | { content: JSXElement; onClick: () => void };
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
  const renderLabel = children(() =>
    props.help ? (
      <LazySpace class="settings-item-content-wrapper" gap="0">
        <LazyLabel weight="semibold" style={labelStyles}>
          {props.label}
        </LazyLabel>

        <LazyTooltip
          content={
            typeof props.help === "object" && "content" in props.help
              ? props.help.content!
              : props.help!
          }
          relationship="description"
          withArrow
        >
          <LazyButton
            icon={<AiOutlineInfoCircle />}
            onClick={
              typeof props.help === "object" && "content" in props.help
                ? props.help.onClick
                : undefined
            }
            shape="circular"
            size="small"
            appearance="transparent"
          />
        </LazyTooltip>
      </LazySpace>
    ) : (
      <LazyLabel weight="semibold" style={labelStyles}>
        {props.label}
      </LazyLabel>
    ),
  );

  const renderContent = children(() => {
    if (props.layout === "vertical") {
      return (
        <LazyFlex direction="column" gap="s" align="stretch">
          {renderLabel()}
          {props.children}
        </LazyFlex>
      );
    }

    const mainContent = (
      <LazyFlex justify="between" align="center">
        {renderLabel()}
        {props.children}
      </LazyFlex>
    );

    if (props.layout === "horizontal" && props.extra) {
      return (
        <LazyFlex direction="column" gap="s" align="stretch">
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

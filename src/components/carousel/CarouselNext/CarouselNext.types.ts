import type { BaseButtonProps } from "~/components/button";
import type { MouseEventHandler, PolymorphicProps } from "~/types";

export type CarouselNextProps = Omit<
  PolymorphicProps<
    "button",
    BaseButtonProps & { onClick?: MouseEventHandler<"button"> }
  >,
  "children"
>;

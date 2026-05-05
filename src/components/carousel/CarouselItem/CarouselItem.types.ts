import type { BaseProps, PolymorphicProps } from "~/types";

interface BaseCarouselItemProps extends BaseProps {
  index?: number;
}

export type CarouselItemProps = PolymorphicProps<
  "fieldset",
  BaseCarouselItemProps,
  false
>;

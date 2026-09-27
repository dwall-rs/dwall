import { createMemo } from "solid-js";
import { useSliderContext } from "../Slider";

export const useSliderIndicator = () => {
  const { min, max, values, orientation } = useSliderContext();

  // /**
  //  * Compute the indicator's start% and end%:
  //  *  - single thumb: from 0 to the current value
  //  *  - dual thumb (range): from the min thumb to the max thumb
  //  */
  // const indicatorStyle = createMemo(() => {
  //   const range = max() - min();
  //   const sorted = values()
  //     .slice()
  //     .sort((a, b) => a - b);
  //   const isSingle = sorted.length === 1;

  //   const startPct = isSingle ? 0 : ((sorted[0] - min()) / range) * 100;
  //   const endPct = ((sorted[sorted.length - 1] - min()) / range) * 100;

  //   if (orientation() === "vertical") {
  //     return {
  //       bottom: `${startPct}%`,
  //       height: `${endPct - startPct}%`,
  //     };
  //   }

  //   return {
  //     left: `${startPct}%`,
  //     width: `${endPct - startPct}%`,
  //   };
  // });

  /**
   * Compute the indicator's start% and end%:
   *  - single thumb: start is fixed at 0, end is the current value's percentage
   *  - multi thumb (range): start is the first thumb, end is the last thumb
   *  Note: values are not sorted, to keep the mapping to thumb indices
   */
  const indicatorStyle = createMemo(() => {
    const range = max() - min();
    const vals = values();
    const isSingle = vals.length === 1;

    const startPct = isSingle ? 0 : ((vals[0] - min()) / range) * 100;
    const endPct = ((vals[vals.length - 1] - min()) / range) * 100;

    if (orientation() === "vertical") {
      return {
        "--relative-size": `${startPct}%`,
        "--start-position": `${endPct - startPct}%`,
      };
    }

    return {
      "--start-position": `${startPct}%`,
      "--relative-size": `${endPct - startPct}%`,
    };
  });

  return { indicatorStyle };
};

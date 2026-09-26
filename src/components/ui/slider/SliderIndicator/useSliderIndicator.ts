import { createMemo } from "solid-js";
import { useSliderContext } from "../Slider";

export const useSliderIndicator = () => {
  const { min, max, values, orientation } = useSliderContext();

  // /**
  //  * 计算 indicator 的 start% 和 end%：
  //  *  - 单 thumb：从 0 到当前值
  //  *  - 双 thumb（range）：从 min thumb 到 max thumb
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
   * 计算 indicator 的 start% 和 end%：
   *  - 单 thumb：start 固定为 0，end 为当前值的百分比
   *  - 多 thumb（range）：start 为第一个 thumb，end 为最后一个 thumb
   *  注意：不对 values 排序，保持与 thumb 索引的对应关系
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

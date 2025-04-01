export const isSubset = <T extends string | number | boolean>(
  subset: T[],
  superset: T[],
): boolean => {
  const supersetSet = new Set(superset);

  return subset.every((item) => supersetSet.has(item));
};

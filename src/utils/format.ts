/** Solar angles are shown to one decimal: precise enough to read, quiet enough for a status line. */
export const formatAngle = (value: number): string => value.toFixed(1);

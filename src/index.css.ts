import { globalStyle, createGlobalTheme } from "@vanilla-extract/css";

export const vars = createGlobalTheme(":root", {
  colors: {
    scrollbar: "rgba(0, 0, 0, 0.1)",
    scrollbarHover: "rgba(0, 0, 0, 0.4)",
  },
});

globalStyle("*", {
  userSelect: "none",
  // @ts-ignore
  WebkitUserDrag: "none",
  MozUserSelect: "none",
  WebkitUserSelect: "none",
  msUserSelect: "none",
});

globalStyle("::-webkit-scrollbar", {
  width: "6px",
  height: "8px",
});

globalStyle("::-webkit-scrollbar-thumb", {
  borderRadius: "10px",
  background: vars.colors.scrollbar,
});

globalStyle("::-webkit-scrollbar-thumb:hover", {
  background: vars.colors.scrollbarHover,
});

globalStyle("::-webkit-scrollbar-track", {
  backgroundColor: "#fff0",
});

globalStyle("body", {
  overflow: "hidden",
  margin: 0,
});

globalStyle("#root", {
  height: "100vh",
  width: "100vw",
  display: "flex",
  flexDirection: "column",
  overflow: "hidden",
});

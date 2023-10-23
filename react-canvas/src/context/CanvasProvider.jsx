import React, { createContext } from "react";

export const CanvasContext = createContext();

export default ({ children, value }) => {
  return (
    <CanvasContext.Provider value={value}>{children}</CanvasContext.Provider>
  );
};

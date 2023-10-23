import React, { useContext } from "react";
import { CanvasContext } from "../context/CanvasProvider";

export default ({ size, x, y }) => {
  const context = useContext(CanvasContext);

  if (context != null) {
    context.beginPath();

    context.rect(x, y, size, size);
    context.stroke();
  }

  return <></>;
};

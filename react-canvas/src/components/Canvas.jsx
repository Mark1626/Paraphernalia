import React, { useEffect, useRef, useState } from "react";
import CanvasProvider from "../context/CanvasProvider.jsx";

export default function({ children }) {
  const [context, setContext] = useState();
  const ref = useRef(null);

  useEffect(() => {
    ref.current.width = 500
    ref.current.height = 500
    const ctx = ref.current.getContext("2d")
    ctx.lineWidth = 1
    ctx.clearRect(0, 0, 500, 500)
    setContext(ctx);
  }, [ref]);

  return (
    <CanvasProvider value={context}>
      <canvas ref={ref} />
      {children}
    </CanvasProvider>
  );
}

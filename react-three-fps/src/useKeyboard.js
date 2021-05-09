import { useState, useEffect } from "react";

const keyMap = {
  "KeyW": "forward",
  "KeyS": "backwards",
  "KeyA": "left",
  "KeyD": "right",
  "Space": "jump"
}

export default () => {
  const [movement, setMovement] = useState({
    forward: false,
    backward: false,
    left: false,
    right: false,
    jump: false,
  });

  useEffect(() => {
    const onKeyDown = (event) => {
      const action = keyMap[event.code];
      if (action) {
        setMovement((state) => ({
          ...state,
          [action]: true,
        }));
      }
    };
    const onKeyUp = (event) => {
      const action = keyMap[event.code];
      if (action) {
        setMovement((state) => ({
          ...state,
          [action]: false,
        }));
      }
    };

    document.addEventListener("keydown", onKeyDown);
    document.addEventListener("keyup", onKeyUp);

    return () => {
      document.removeEventListener("keydown", onKeyDown);
      document.removeEventListener("keyup", onKeyUp);
    };
  }, []);

  return movement;
};

import React, { useState, useEffect } from "react";
import { ThemeProvider } from "styled-components";
import { Background } from "./App.style";

const ThemeSwitch = ({theme, setTheme}) => {
  const handleThemeChange = (e) => {
    setTheme(e.target.value);
  };

  return (
    <select value={theme} onChange={handleThemeChange}>
      <option value="light">Light</option>
      <option value="dark">Dark</option>
      <option value="my-eyes">My Eyes</option>
    </select>
  );
};

export default () => {
  const [theme, setTheme] = useState("dark");

  useEffect(() => {
    console.log(theme)
  }, [theme])

  return (
    <>
      <ThemeSwitch theme={theme} setTheme={setTheme} />
      <ThemeProvider theme={{ mode: theme }}>
        <Background></Background>
      </ThemeProvider>
    </>
  );
};

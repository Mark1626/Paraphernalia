import styled from 'styled-components'
import theme from 'styled-theming'

const bgColor = theme('mode', {
  light: "#fff",
  dark: "#000",
});

export const Background = styled.div`
  background-color: ${bgColor};
  width: 100px;
  height: 100px;
  border: 2px solid red;
`

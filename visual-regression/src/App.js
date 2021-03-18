import React, { useState } from 'react';
import styled from 'styled-components';
import Button from './components/Button';

const Container = styled.div`
  width: fit-content;
  padding: 10px;
`

export default () => {

  const [count, setState] = useState(0);
  const increment = () => {
    setState(count + 1);
  }
  const decrement = () => {
    setState(count - 1);
  }

  return (
    <>
      <Container>
        <span>Count is {count}</span>
        <Button name="Increment" action={increment} />
        <Button name="Decrement" action={decrement} />
      </Container>
    </>
  );
}

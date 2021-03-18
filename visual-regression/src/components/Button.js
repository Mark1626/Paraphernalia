import React from "react";
import styled from "styled-components";

const StyledButton = styled.div`
  width: 100px;
  padding: 5px;
  border: 2px solid;
  border-radius: 10px;
  background-color: #eee;
  text-align: center;
  line-height: 50px;

  &:hover {
    cursor: pointer;
  }
`;

export default ({ name, action }) => {
  return (
    <>
      <StyledButton datatest-id="button" onClick={action}>
        {name}
      </StyledButton>
    </>
  );
};

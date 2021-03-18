import React from 'react'
import Button from "./Button";
import renderer from 'react-test-renderer';
import { render, fireEvent, screen } from "@testing-library/react";

describe("test button", () => {
  // it("should be able to render", () => {
  //   const mockFn = jest.fn();

  //   const {asFragment} = renderer(<Button name="test" action={mockFn} />);

  //   expect(asFragment()).toMatchSnapshot()
  // })

  it("should be able to render", () => {
    const mockFn = jest.fn();

    const tree = renderer.create(<Button name="test" action={mockFn} />).toJSON();

    expect(tree).toMatchSnapshot()
  })

  it("should be able to click a button", async () => {
    const mockFn = jest.fn();

    render(<Button name="test" action={mockFn} />);

    fireEvent.click(screen.getByText("test"));

    expect(mockFn).toHaveBeenCalled();
  });

});

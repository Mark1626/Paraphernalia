# React + Canvas

![Square Matrix](https://cdn.glitch.com/625f2780-034f-4860-8a83-6757cf3f36f4%2Fsquare_matrix.png?v=1619610818982)

## What is this?

Manipulating the canvas is a mutable task, so performing this in react 
would require proper handling to prevent multiple rerenders

## How do I use canvas then

Example of drawing in the canvas

```js
const Canvas = () => {
  const ref = useRef()
  
  useEffect(() => {
    const context = ref.current.getContext("2d")
    
    context.beginPath();
    context.lineTo(10, 10);
    context.fill();
  }, [ref])

  return <canvas ref={ref}>
}
```

## Isn't this needlessly complex?

Yes, though now since the canvas can be represented as a component, I can store the canvas's context 
in a React Context and reuse it

```js
// Canvas
export default ({ children }) => {
  const [context, setContext] = useState();
  const ref = useRef(null);

  useEffect(() => {
    setContext(ref.current.getContext("2d"));
  }, [ref]);

  return (
    <CanvasProvider value={context}>
      <canvas ref={ref} />
      {children}
    </CanvasProvider>
  );
}

// Square.js
export default ({ length, x, y }) => {
  const context = useContext(CanvasContext);

  if (context != null) {
    context.beginPath();
    context.rect(x, y, length, length);
    context.stroke();
  }
  
  return <></>;
};

// home.jsx
<Canvas>
  <Square length={40} x={0} y={0} />
</Canvas>
```

## How about animations?

I'll cover this in another example

You can see the previous example [live here](https://react-canvas-animation.glitch.me)

The source [here](https://glitch.com/edit/#!/react-canvas-animation)

## Credits

I used [this article](https://thibaut.io/react-canvas-components) as reference. My idea is to 
get more people aware of this concept and the extensibility you can achieve with this

The image I drew is from an example in [generativeartistry](https://generativeartistry.com/tutorials/hypnotic-squares/) by [tholman](http://tholman.com/)

const compose = (...fns) =>
  fns.reduceRight(
    (prevFn, nextFn) => (...args) => nextFn(prevFn(...args)),
    (value) => value // Unity function
  );

const Unit = (original) => {
  const {
    update
  } = original

  return {
    ...original,
    update() {
      update()
      console.log("unit")
    }
  }
}

const A = compose(
  Unit
)({
  state: {
    x: 5
  },
  update: () => console.log("test")
})

console.log(A.state)
console.log(A.update())

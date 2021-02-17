# Hashcash Proof of Work

Based on this [implementation](https://github.com/hashcash-org/hashcash)

Implementation uses `BigInt` so it may be vulnerable to Timing attack

### Usage

```js
const stamp = mintCash('test@abc.com', 20)
```

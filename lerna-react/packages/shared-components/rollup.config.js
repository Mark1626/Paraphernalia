import resolve from '@rollup/plugin-node-resolve';
import babel from '@rollup/plugin-babel';

export default {
  input: './src/index.js',
  external: [
    'react'
  ],
  plugins: [
    resolve(),
    babel({
      exclude: 'node_modules/**',
    })
  ],
  output: {
    file: 'lib/bundle.js',
    formats: ['cjs', 'es'],
    sourceMap: true
  }
};
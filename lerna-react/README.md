# React Monorepo with Lerna

### Setup

#### Setup Lerna

- `yarn add lerna -D`
- `yarn lerna init`

#### Setup Shared Component

- `mkdir packages/shared-components`
- `cd packages/shared-components && yarn init`

#### Setup React App

- `cd packages`
- `yarn create react-app main-app`
- Add `shared-component` as a dependency in `packages.json`

#### Bootstrap both projects in lerna

- `yarn lerna bootstrap`

### Build steps

- **The `shared-components` module needs to be build before it can be consumed by the `main-app`**

#### Babel

- One way to do this is to have a script in `shared-components/packace.json` that does a `babel src -o lib`
- This is specified in the script in `package.json` `babel-build`

#### Rollup

- `rollup` can be used to bundle the files
- Define a `rollup.config.js` in `shared-components`
- Bundle the project with `rollup -c`
- This is specified in the script in `package.json` `bundle`

### Running the app with lerna

- `yarn lerna run bundle`
- `yarn lerna run start`

### Running the app simplified

- `yarn start`

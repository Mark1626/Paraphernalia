# Spike on visual-regression with loki

### Storybook setup

- `px sb init`
- Change route to your stories in `.storybook/main.js`

### Loki Setup

- `yarn add loki -D`
- `yarn loki init`
- `yarn loki update`

## Running visual regression

- Start storybook `yarn storybook`
- (Optional) If using chrome in docker, start docker
- `yarn loki test`

## Review and Approving snapshot

- Difference is present in `.loki/reference`
- `yarn loki approve`

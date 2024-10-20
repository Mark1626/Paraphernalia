# Autocompletion of Yaml Path

This example demonstrates autocompletion for a executable with `test [run/diagnose] [options`.

`test run` subcommand has two options `test run --path <file-path>` and `test run --set <yaml-path>`. For the `--set` option the yaml is autocompleted , for the `--config` normal file autocompletion is done.

## Bash

Source the script to enable the completions

```
source bash-completions.bash
```

## Zsh

The same completion script can be reused in zsh using `bashcompinit`

```
source bash-completions.bash
bashcompinit
```

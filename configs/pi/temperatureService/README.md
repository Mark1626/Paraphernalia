# Temperature Service

A small service to monitor the temperature of my raspberry pi

## Usage

- Run `tmp_monitor.sh` as a `cron` to create the temperature log
- Serve the contents of the folder
- Access `index.html` to see the graph

#### Cron Tab Example

```sh
* * * * * sh /path/to/temperature.sh
```

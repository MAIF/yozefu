Work in progress

|                           | Default behavior                  |      CLI option | Environment variable |           Configuration file |
| ------------------------- | --------------------------------- | --------------: | -------------------: | ---------------------------: |
| Workspace (or config dir) | `~/.config/io.maif.yozefu/`       |  `--config-dir` |  `YOZEFU_CONFIG_DIR` |                           No |
| Configuration file        | `${workspace}/config.json`        | `--config-file` |                  N/A |                           No |
| Log file                  | `${workspace}/application.log`    |    `--log-file` |    `YOZEFU_LOG_FILE` |        jsonpath  `/log_file` |
| Export directory          | `$PWD/export-{datetime-now}.json` |      `--output` |                   No | jsonpath `/export_directory` |

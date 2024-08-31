Elephantine
---

A pinentry program that allows shelling out to an arbitrary program to get user input.
We aim to implement enough of the Assuan protocol to support this use case.

# Usage
```
Implements the pinentry protocol and uses walker for PIN input

Usage: elephantine [OPTIONS]

Options:
  -d, --debug...                       The debug level [env: ELEPHANTINE_DEBUG=]
      --config-file <FILE>             Path to the configuration file [env: ELEPHANTINE_CONFIG_FILE=] [default: /home/narthana/.config/elephantine/elephantine.toml]
  -D, --display [<DISPLAY>]            The X display to use for the dialog [env: PINENTRY_DISPLAY=]
  -T, --ttyname [<FILE>]               The tty terminal node name [env: TTYNAME=]
  -N, --ttytype [<NAME>]               [env: TTYTYPE=]
  -C, --lc-ctype [<STRING>]            The `LC_CTYPE` locale category [env: LC_CTYPE=]
  -M, --lc-messages [<STRING>]         The `LC_MESSAGES` value [env: LC_MESSAGES=]
  -o, --timeout [<SECS>]               Timeout in seconds for requests that show dialogs to the user. E.g. GETPIN, CONFIRM, etc [env: ELEPHANTINE_TIMEOUT=] [default: 300]
  -g, --no-local-grab <NO_LOCAL_GRAB>  Grab keyboard only while the window is focused [env: ELEPHANTINE_NO_LOCAL_GRAB=] [possible values: true, false]
  -W, --parent-wid [<WINDOW_ID>]       Parent window ID (for partitioning)
  -c, --colors [<STRING>]              Custom colors for the dialog
  -a, --ttyalert [<STRING>]            The alert mode (none, beep, or flash)
      --command <COMMAND>...           The command to run the dialog. It must print the input to stdout [default: "walker --password"]
  -h, --help                           Print help
  -V, --version                        Print version
```

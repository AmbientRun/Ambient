# Settings

Ambient supports a number of settings that can be configured using the `settings.toml` file. This file is located under the platform's config directory, e.g. `C:\Users\*USER*\AppData\Roaming\Ambient\Ambient\config\settings.toml` on Windows.

## Settings

[general]\
sentry = [bool, String] # [enabled, DSN]

[render]\
resolution = [int, int]\
vsync = bool\
render_mode = String\ # "MultiIndirect", "Indirect", "Direct"\
software_culling = bool

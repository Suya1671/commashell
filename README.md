# Commashell

## A GUI Shell of all time

Commashell is a GTK + Libadwaita based shell for Linux powered by [Astal](https://aylur.github.io/astal/).

NOTE: current designed with [commafiles](https://github.com/suyashtnt/commafiles) in mind only. I will add support for usage with other rices soon. Many things will not work
or look wrong if you try to use it with other rices right now.

## Features

- top status bar (`astal toggle top`)
  - [ ] Weather
  - [ ] Network status
  - [x] Time
  - [x] Wallpaper switcher (requires [commafiles](https://github.com/suyashtnt/commafiles))
- right music bar (`astal toggle right`)
  - [x] Music controls (Uses MPRIS)
  - [x] Lyrics (requires [sptlrx](https://github.com/raitonoberu/sptlrx))
  - [x] Music visualizer (uses [CAVA](https://github.com/karlstav/cava))
- Notifications
  - Uses regular wayland protocols for notifications
- Launcher
  - [x] Fuzzy app launcher
  - [x] Calculator (requires [libqalculate](https://qalculate.github.io/). `= ` prefix)
  - [x] Journal entry (requires Obsidian + Thino Pro plugin. I will add support for other journaling apps soon:tm:)
  - [ ] Task taking (will use ticktick. I will add support for other task managers soon:tm:)

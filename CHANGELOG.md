# Changelog

All notable changes to this project are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the project uses [Semantic Versioning](https://semver.org/).

The release workflow copies the section for each version into its GitHub release,
so add a section here before bumping the version in `Cargo.toml`.

## [1.0.0] - 2026-10-23

### Added
- Tray icon with **Dev Mode** and **Play Mode**; the icon, tooltip and menu checkmark show the current mode.
- Play Mode stops Docker Desktop, shuts down WSL and politely closes the apps listed in `close-apps.txt`, then reports how much RAM was freed.
- Dev Mode starts Docker Desktop and notifies you when the engine is ready.
- `cleanup` command (and menu item) that prunes dangling Docker images and week-old build cache, trims the Docker disk, and verifies the npm cache. Never touches containers or volumes, and never starts Docker.
- "Start with Windows" toggle and "Edit apps to close" menu items.
- Command-line modes: `devplay.exe play`, `devplay.exe dev`, `devplay.exe cleanup` for shortcuts and scheduled tasks. Unknown commands are written to the log.
- Custom app and tray icons, embedded in the exe.
- Single-instance guard, tray icon restored after Explorer restarts, actions logged to `devplay.log`.

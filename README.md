<p align="center">
  <img src="assets/devplay.png" width="128" height="128" alt="DevPlay icon">
</p>

<h1 align="center">DevPlay</h1>

<p align="center">A tiny Windows tray app that switches your PC between <b>Dev Mode</b> and <b>Play Mode</b>.</p>

Developer tools like Docker Desktop and WSL quietly hold gigabytes of RAM. On a low-end PC, especially one with integrated graphics that borrows system memory, that RAM is the difference between a playable game and a stuttering one. DevPlay frees it with one click and brings your dev environment back with another.

- **Tiny:** ~400 KB exe, **under 1 MB of RAM** while sitting in the tray. Raw Win32 via [`windows-sys`](https://crates.io/crates/windows-sys), no GUI framework, no runtime.
- **Safe:** never deletes containers or volumes, never changes Windows settings.

## What it does

| Menu item | Action |
|---|---|
| **Play Mode** | Stops Docker Desktop, runs `wsl --shutdown`, and asks the apps in `close-apps.txt` to close (like clicking X, so they can save). Shows how much RAM was freed. |
| **Dev Mode** | Starts Docker Desktop and notifies you when the engine is ready. |
| **Run cleanup now** | If Docker is running: prunes dangling images and build cache older than 7 days, then trims the Docker disk so a sparse VHDX hands space back to Windows. Also runs `npm cache verify`. |
| **Edit apps to close...** | Opens `close-apps.txt` in Notepad. |
| **Start with Windows** | Adds or removes DevPlay from your startup apps. |

Left- or right-click the tray icon to open the menu. The icon shows the current mode: a sky-blue play button for Dev Mode, amber for Play Mode.

## Install

1. Download `devplay.exe` from the [latest release](https://github.com/mahmoudalnkeeb/devplay/releases/latest).
2. Put it in a folder of your choice, e.g. `D:\Tools\DevPlay\`. Its config (`close-apps.txt`) and log (`devplay.log`) are created next to it.
3. Run it. The icon appears in the system tray (you may need to click the `^` arrow).

The exe isn't code-signed, so Windows SmartScreen may warn on first run: click **More info → Run anyway**. Each release includes `SHA256SUMS.txt` so you can check the download:

```powershell
Get-FileHash .\devplay.exe -Algorithm SHA256
```

## Command line

The same actions run without the tray, which is handy for shortcuts and Task Scheduler:

```powershell
devplay.exe play      # Play Mode once, then exit
devplay.exe dev       # Dev Mode once, then exit
devplay.exe cleanup   # cleanup once, then exit
```

Example: run cleanup weekly on Sundays at 8 PM.

```powershell
$action  = New-ScheduledTaskAction -Execute 'D:\Tools\DevPlay\devplay.exe' -Argument 'cleanup'
$trigger = New-ScheduledTaskTrigger -Weekly -DaysOfWeek Sunday -At 8pm
Register-ScheduledTask -TaskName 'DevPlay Cleanup' -Action $action -Trigger $trigger
```

## Configuration

`close-apps.txt` lists the processes Play Mode closes, one per line:

```text
# Lines starting with # are ignored.
brave.exe
WhatsApp.Root.exe
Code.exe
```

## Requirements

- Windows 10 or 11 (x64).
- Docker Desktop and WSL are optional; the related steps are skipped if they aren't installed.

## Build from source

```powershell
cargo build --release
```

The exe lands in `target\release\devplay.exe`. `build.rs` uses [`winresource`](https://crates.io/crates/winresource) to embed the app icon and version info, which needs a resource compiler: `rc.exe` from the Windows SDK (MSVC toolchain, used for releases) or MinGW's `windres` (GNU toolchain). Release builds fail without one; debug builds, tests and clippy skip the resources with a warning.

The icons are drawn in code by [`assets/generate-icons.ps1`](assets/generate-icons.ps1). Edit it and rerun to change them:

```powershell
powershell -ExecutionPolicy Bypass -File assets\generate-icons.ps1
```

### Project layout

```
src/
  main.rs       entry point: tray, or a one-shot command
  actions.rs    Play / Dev / Cleanup and their reports
  config.rs     close-apps.txt
  docker.rs     Docker Desktop control
  log.rs        devplay.log
  sys/          Win32 wrappers (process, memory, disk, time, autostart, UTF-16)
  tray/         window, menu, notification icon, tray icons, shared state
build.rs        embeds the app icon and version info
```

## Releasing

Releases are automated by [`.github/workflows/release.yml`](.github/workflows/release.yml):

1. Add a `## [x.y.z] - YYYY-MM-DD` section to [`CHANGELOG.md`](CHANGELOG.md).
2. Bump `version` in `Cargo.toml` (and run `cargo build` so `Cargo.lock` updates).
3. Commit and push to `main`.

When `main` has a version with no matching release, the workflow runs the tests, builds the exe, tags `vx.y.z`, and publishes a GitHub release with that changelog section, the exe, a zip, and SHA-256 checksums.

## License

[MIT](LICENSE)

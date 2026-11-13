<p align="center">
  <img src="assets/app-icons/flowapps-symbol.png" width="128" height="128" alt="FlowApps">
</p>

<h1 align="center">FlowApps</h1>

<p align="center">
  <b>One place to install, update and open every app in the FlowApps suite.</b><br>
  Flowgram, Voxflow and Mailflow: the right build for your computer, kept up to date.
</p>

<p align="center">
  <a href="https://github.com/reachneoagent-commits/FlowApps-Launcher/releases/latest"><b>Download FlowApps</b></a>
</p>

<p align="center">
  <img src="docs/screenshots/flowapps-launcher-grid.png" alt="FlowApps on macOS: a card for each app with its icon, version and an Install or Open button" width="900">
</p>

## What it does

The FlowApps apps each ship their own builds for every platform. FlowApps saves you from
hunting through release pages: it lists every app in the suite, shows which ones you have
and which version, installs the right build for your operating system and processor, and tells
you when an update is out. It is a small native app that can sit in your menu bar or system tray.

## Features

<table>
  <tr>
    <td width="33%" valign="top">
      <h3>📦 Install in one click</h3>
      Picks the right download for your OS and CPU (DMG, MSI, AppImage, .deb, .rpm and more), checks it against the release's SHA-256 checksums and installs it.
    </td>
    <td width="33%" valign="top">
      <h3>🔄 Stay up to date</h3>
      Checks each app's GitHub releases and updates the ones you have. A notification tells you when there's something new.
    </td>
    <td width="33%" valign="top">
      <h3>🧭 Always at hand</h3>
      Opens any FlowApps app from one window, or from the tray and menu-bar icon. Native egui, no Electron, no web view.
    </td>
  </tr>
</table>

The web version shows the same catalog in the browser, with the latest download for each app.

## Downloads

Download the build for your computer from the [latest release](https://github.com/reachneoagent-commits/FlowApps-Launcher/releases/latest), or browse [all releases](https://github.com/reachneoagent-commits/FlowApps-Launcher/releases). Every release has a `SHA256SUMS.txt` to check the files against. `<ver>` is the version number, for example `0.1.0`.

**Windows** (10 and 11)

| File | What it is |
|---|---|
| `flowapps-launcher-<ver>-windows-x64.msi` | Installer for most PCs (Intel and AMD, 64-bit) |
| `flowapps-launcher-<ver>-windows-arm64.msi` | Installer for Windows on Arm |
| `flowapps-launcher-<ver>-windows-x86.msi` | Installer for 32-bit Windows |
| `flowapps-launcher-<ver>-windows-{x64,arm64,x86}-portable.zip` | No install: unzip and run `flowapps-launcher.exe` |

**macOS** (11 Big Sur or later)

| File | What it is |
|---|---|
| `flowapps-launcher-<ver>-macos-universal.dmg` | Universal app for Apple silicon and Intel Macs. Drag FlowApps to Applications |

**Linux** (x86_64 and aarch64; glibc 2.35 or newer)

| File | What it is |
|---|---|
| `flowapps-launcher-<ver>-linux-{x86_64,aarch64}.AppImage` | Runs on almost any distro without installing |
| `flowapps-launcher-<ver>-linux-{x86_64,aarch64}.deb` | Debian, Ubuntu, Mint, Pop!_OS: `sudo apt install ./flowapps-launcher-…deb` |
| `flowapps-launcher-<ver>-linux-{x86_64,aarch64}.rpm` | Fedora, openSUSE, RHEL: `sudo dnf install ./flowapps-launcher-…rpm` |
| `flowapps-launcher-<ver>-linux-{x86_64,aarch64}.flatpak` | Flatpak bundle |
| `flowapps-launcher-<ver>-linux-{x86_64,aarch64}.tar.gz` | Plain tree (`bin/`, `share/`) |

## The FlowApps suite

| App | What it's for | GitHub |
|---|---|---|
| **Flowgram** | Grammar and spell checker for Mac and Windows | [GitHub](https://github.com/reachneoagent-commits/flowgram) |
| **Voxflow** | Offline speech-to-text app | [GitHub](https://github.com/reachneoagent-commits/voxflow) |
| **Mailflow** | Mail, calendar, contacts and tasks app, with optional AI writing help | [GitHub](https://github.com/reachneoagent-commits/mailflow) |

## License and credits

FlowApps is forked from [ArtCraft Launcher](https://github.com/storytold/craft-launcher) by the ArtCraft team.
The install/update/uninstall engine, catalog format and UI patterns are adapted from ArtCraft Launcher,
which is dual-licensed under MIT or Apache-2.0. FlowApps is itself dual-licensed under
[MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.

**Copyright (c) 2026 Reach Neo.** Required notices are in [NOTICE](NOTICE).

Bundled fonts, icons, images and other assets keep their own open licenses; each one is listed
with its author, source and license in [ATTRIBUTION.md](ATTRIBUTION.md).

The app icons of the FlowApps apps (`assets/app-icons/`) are each app's own icon, copied from its
repository and licensed like that app (MIT OR Apache-2.0).

**The FlowApps name, logo and symbol are trademarks of Reach Neo.** They may not be used without
permission. The ArtCraft name, logo and "A" mark remain trademarks of the ArtCraft Team and are
not used in FlowApps.

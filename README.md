<p align="left">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/header-dark.png">
    <img src="assets/header-light.png" width="600" height="160" alt="FlowApps logo and name">
  </picture>
</p>

<p align="center">
  <b>One place to install, update and open every app in the FlowApps suite.</b><br>
  Gramflow, Voxflow and Mailflow: the right build for your computer, kept up to date.
</p>

<p align="center">
  <a href="https://github.com/reachneoagent-commits/FlowApps-Launcher/releases/latest"><b>Download FlowApps</b></a>
</p>

## Why FlowApps?

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

## Download and install

Download FlowApps for your system (latest version, 0.20.10), then open it to install Gramflow, Voxflow and Mailflow.

| Platform | Installer |
| --- | --- |
| macOS (Apple silicon and Intel) | [DMG](https://github.com/reachneoagent-commits/FlowApps-Launcher/releases/download/v0.20.10/flowapps-launcher-0.20.10-macos-universal.dmg) |
| Windows x64 | [Installer](https://github.com/reachneoagent-commits/FlowApps-Launcher/releases/download/v0.20.10/flowapps-launcher-0.20.10-windows-x64.msi) · [Portable](https://github.com/reachneoagent-commits/FlowApps-Launcher/releases/download/v0.20.10/flowapps-launcher-0.20.10-windows-x64-portable.zip) |
| Windows ARM64 | [Installer](https://github.com/reachneoagent-commits/FlowApps-Launcher/releases/download/v0.20.10/flowapps-launcher-0.20.10-windows-arm64.msi) · [Portable](https://github.com/reachneoagent-commits/FlowApps-Launcher/releases/download/v0.20.10/flowapps-launcher-0.20.10-windows-arm64-portable.zip) |

Older versions are on the [Releases page](https://github.com/reachneoagent-commits/FlowApps-Launcher/releases). Every release has a `SHA256SUMS.txt` to check the files against.

FlowApps isn't signed with a paid Apple or Microsoft certificate, so your computer will warn you the first time you open it.

- **Mac:** open the DMG and drag FlowApps to Applications. Then right-click FlowApps, choose Open, and click Open again.
- **Windows:** if SmartScreen appears, click More info, then Run anyway.

## The FlowApps suite

| App | What it's for | GitHub |
|---|---|---|
| **Gramflow** | Grammar and spell checker for Mac and Windows | [GitHub](https://github.com/reachneoagent-commits/gramflow) |
| **Voxflow** | Offline speech-to-text app | [GitHub](https://github.com/reachneoagent-commits/voxflow) |
| **Mailflow** | Mail, calendar, contacts and tasks app, with optional AI writing help | [GitHub](https://github.com/reachneoagent-commits/mailflow) |
| **TVflow** | IPTV player for Mac, Windows and your TV | [GitHub](https://github.com/reachneoagent-commits/tvflow) |
| **Writeflow** | Scan and rewrite AI-sounding text, privately | [GitHub](https://github.com/reachneoagent-commits/writeflow) |
| **Buildflow** | See how software works, then build your own with AI | [GitHub](https://github.com/reachneoagent-commits/buildflow) |
| **VPNFlow** | Free WireGuard VPN with leak checks (preview) | [GitHub](https://github.com/reachneoagent-commits/vpnflow) |
| **Irisflow** | Free, open source video meetings (preview) | [GitHub](https://github.com/reachneoagent-commits/irisflow) |

## Supported platforms

- **Desktop:** macOS (Apple silicon and Intel) and Windows (x64 and ARM64)
- **Web:** the same catalog runs in a browser

## Build it yourself

You need [Rust](https://rustup.rs/). Then run:

```bash
cargo build --release -p flowapps-launcher
```

## About this fork

FlowApps is forked from [ArtCraft Launcher](https://github.com/storytold/craft-launcher) by the ArtCraft team.
The install/update/uninstall engine, catalog format and UI patterns are adapted from ArtCraft Launcher,
which is dual-licensed under MIT or Apache-2.0. Required notices are in [NOTICE](NOTICE).

### Credits

- **[ArtCraft Launcher](https://github.com/storytold/craft-launcher)** by the ArtCraft team: the install, update and uninstall engine, catalog format and UI patterns (MIT or Apache-2.0).
- **App icons** in `assets/app-icons/` are each app's own icon, copied from its repository and licensed like that app (MIT OR Apache-2.0).
- **Bundled fonts, icons, images and other assets** keep their own open licenses; each one is listed with its author, source and license in [ATTRIBUTION.md](ATTRIBUTION.md).

## License

FlowApps is dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.

**Copyright (c) 2026 Reach Neo.**

**The FlowApps name, logo and symbol are trademarks of Reach Neo.** They may not be used without
permission. The ArtCraft name, logo and "A" mark remain trademarks of the ArtCraft Team and are
not used in FlowApps.

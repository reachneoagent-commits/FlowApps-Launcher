# AGENTS.md: guide for contributors

FlowApps installs, updates and launches the FlowApps suite: **Gramflow**, **Voxflow** and **Mailflow**,
each hosted in a repository under https://github.com/reachneoagent-commits. It is written in Rust
only: the desktop app is native egui/eframe, and the web build is the same Rust compiled to
WebAssembly. FlowApps is forked from ArtCraft Launcher
(https://github.com/storytold/craft-launcher); see `NOTICE`.

## Layout

```text
crates/core      pure, wasm-safe: the app catalog (crates/core/catalog.toml), versions, GitHub
                 release parsing, picking the right download per platform
crates/native    desktop only: HTTP, detecting installed apps, download, verify and install
crates/ui-egui   the shared egui UI (desktop and web)
apps/            desktop shell (flowapps-launcher) and web build (flowapps-launcher-web)
packaging/       macOS, Windows, Linux and web packaging scripts
brand/           FlowApps logo and icon sources
```

## Rules

- **Never crash.** Non-test code does not use `unwrap`, `expect`, `panic!` or `unsafe`. Everything
  from the network is untrusted.
- **Install only from the catalog.** Downloads come from `https://github.com/<repo>/releases` for
  the repos in `crates/core/catalog.toml`, and are checked against the release's SHA-256 before
  anything is installed. Mac builds that aren't Apple-signed use `mac_team = "unsigned"`.
- **Strings go through `crates/ui-egui/src/i18n`** (`tl!("...")`); English is the key.
- **Assets need a row in `ATTRIBUTION.md`.**

## Adding an app

Add a `[[apps]]` block to `crates/core/catalog.toml` (repo, bundle id, publisher), its 128 px icon
to `assets/app-icons/`, an arm in `app_icon_png` in `crates/core/src/catalog.rs`, and make sure its
release files are named like `<name>-<version>-<os>-<arch>.<ext>` or Tauri's
`<Name>_<version>_<arch>.<ext>` (see `crates/core/src/platform.rs`).

## Releasing

The version lives in `[workspace.package]` in the root `Cargo.toml`. Run the **Release** workflow
from the Actions tab: it builds the macOS DMG and the Windows MSI and portable zip (x64 and
arm64), writes `SHA256SUMS.txt` and publishes `v<version>`.
Before finishing a change: `cargo build --release -p flowapps-launcher` and the tests you touched.

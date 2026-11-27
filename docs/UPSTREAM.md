# Keeping up with upstream

The FlowApps Launcher is a fork of [ArtCraft Launcher](https://github.com/storytold/craft-launcher)
(MIT or Apache-2.0). This is how we take their updates without losing our name, links, catalog or
features.

## Two kinds of change

1. **Branding** in upstream's files is made **only by `rebrand/apply.py`**, from
   `rebrand/brand.json`. Never hand-edit it: add an entry and run the script.
   - `tokens`: names changed everywhere: crates `artcraft_launcher*` → `flowapps_launcher*` and
     environment variables `ARTCRAFT_LAUNCHER_*` → `FLOWAPPS_LAUNCHER_*`.
   - `edits`: names, ids (`ai.reachneo.flowapps`), links, publisher and installer strings on single
     lines (about 500, generated from our history, longest first per file).
   - `prepend` / `append`: the agent note in `CLAUDE.md`, `.gitignore` lines, merge rules in
     `.gitattributes`.
   `apply.py` is the same engine as in the other FlowApps forks; this copy adds `tokens` and
   `icons_dir`, which older copies ignore.
2. **Features** are ordinary commits that git carries through a merge: the FlowApps catalog, the
   release-candidate channel, self-update (`crates/update`, vendored from craft-libs), Settings, the
   tray and header without ArtCraft's Discord, website and social links, and the app list UI.

So **do not blanket "take theirs"** on a conflicted file unless it carries branding only.

## Renamed paths

We renamed `apps/artcraft-launcher*` → `apps/flowapps-launcher*`, the app id
`ai.storyteller.artcraft-launcher` → `ai.reachneo.flowapps` in `packaging/`, and the icon files in
`assets/app-icon/`. Git's rename detection carries upstream's changes to those files across; a file
upstream adds under an old directory name lands in the renamed one. Check `git status` for a new
file left under an `artcraft` path and move it by hand.

## What lives where

| Ours (keep ours in a conflict) | Branding only (take theirs, then run `apply.py`) | Branding and features (resolve by hand, then run `apply.py`) |
|---|---|---|
| `rebrand/`, `brand/`, `docs/UPSTREAM.md`, `README.md`, `NOTICE`, `AGENTS.md`, `ATTRIBUTION.md` | everything else that came from upstream | `Cargo.toml`, `apps/flowapps-launcher/src/main.rs`, `tray.rs` |
| `crates/core/catalog.toml`, `crates/update/` | | `crates/core/src/`: `catalog.rs`, `details.rs`, `feed.rs`, `hosted_catalog.rs`, `platform.rs`, `protocol.rs`, `version.rs` |
| `assets/app-icon/`, `assets/app-icons/`, `screenshots/`, `docs/brand/` | | `crates/native/src/`: `backend.rs`, `desktop_entry.rs`, `detect.rs`, `hosted_catalog.rs`, `install.rs`, `install/macos_verify.rs`, `login.rs`, `packages.rs`, `pins.rs`, `process.rs`, `screenshots.rs`, `single_instance.rs`, `source.rs` |
| `.github/workflows/` (ours replace theirs) | | `crates/ui-egui/src/`: `app.rs`, `bar.rs`, `card_parts.rs`, `details.rs`, `dialogs.rs`, `icons.rs`, `model.rs`, `settings_dialog.rs`, `theme.rs`, `versions.rs`, `views.rs` |
| `crates/i18n/` translations (ours; merge their new strings by hand) | | `crates/ui-egui/tests/*.rs`, `packaging/macos/dmg/`, `packaging/windows/package.ps1` |

## Merging upstream

Set the clone up once, so upstream's tags arrive as `upstream/v…` and never collide with ours:

```bash
git remote add upstream https://github.com/storytold/craft-launcher.git
git config remote.upstream.tagOpt --no-tags
git config --add remote.upstream.fetch '+refs/tags/*:refs/tags/upstream/*'
git config merge.ours.driver true
```

Never push the `upstream/*` tags. Then:

```bash
git fetch upstream
git checkout -b upstream-sync
git merge upstream/main
```

Resolve conflicts by the table above, then re-apply the branding and check it:

```bash
python3 rebrand/apply.py
python3 rebrand/apply.py --check     # prints nothing and exits 0 when everything is rebranded
git grep -n -i 'artcraft\|storytold\|discord' -- apps crates packaging   # read what is new
```

`--check` only knows the lines it was taught. Upstream's new ArtCraft names, links or Discord
buttons show up in the `git grep`: add an edit (or a token) for each to `rebrand/brand.json`. If the
script stops with `could not find … to edit`, upstream changed that line: find where the text went
and update the entry.

Build and test (`cargo xtask ci`), run the launcher once and look at it, commit and open a pull
request.

`rebrand/apply.py` was checked on 2026-11-26 against upstream `66beafa` (the merge-base): taking
upstream's version of every branding-only file and re-running the script left no upstream branding.

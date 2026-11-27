<!-- FlowApps Launcher (rebrand/apply.py) -->
# FlowApps Launcher

This repository is the **FlowApps Launcher**, a fork of ArtCraft Launcher. Read `docs/UPSTREAM.md` first. In short:

- Never hand-edit a file that came from upstream to change branding, ids, links or icons: add the
  change to `rebrand/brand.json` and run `python3 rebrand/apply.py`, so it survives the next upstream merge.
- FlowApps features (the catalog, Settings, self-update, release-candidate channel) are ordinary
  commits: in a merge conflict keep both sides by hand, never blanket "take theirs".
- Crates are `flowapps_launcher*`, environment variables `FLOWAPPS_LAUNCHER_*` (the "tokens" in brand.json).

@AGENTS.md

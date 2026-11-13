//! The FlowApps suite: bundled data with a signed hosted startup snapshot.
//!
//! The list is data so a later version can fetch it from an API; [`SUITE`] is the built-in copy.
//!
//! Text here is English, the source the UI translates from: each `tagline` has a catalog row
//! keyed by the app id (`tr_ctx(app.id, app.tagline)` in `ui-egui`).

/// Who makes an app. Official apps live under [`GITHUB_OWNER`]; a community app is maintained
/// outside ArtCraft and is labelled as such wherever it is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppKind {
    Official,
    Community,
}

/// One app in the suite.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CraftApp {
    /// Machine name: repo, binary, package and asset stem (`photocraft`).
    pub id: &'static str,
    /// Display name (`PhotoCraft`).
    pub name: &'static str,
    /// What it's for, in a few words (English; the UI translates it by app id).
    pub tagline: &'static str,
    /// Brand colour (sRGB), for surfaces: getartcraft.com's `--app`.
    pub color: [u8; 3],
    /// The brand colour for text on a light background (the site's light `--app-ink`).
    pub ink_light: [u8; 3],
    /// The brand colour for text on a dark background (the site's dark `--app-ink`).
    pub ink_dark: [u8; 3],
    /// GitHub `owner/repo`.
    pub repo: &'static str,
    /// Older names its release assets may carry (PdfCraft shipped as PrintCraft until 0.2.1).
    pub former_ids: &'static [&'static str],
    /// Older display names its installers registered (PdfCraft's MSI said PrintCraft until 0.2.1).
    pub former_names: &'static [&'static str],
    /// The `.app` bundle stem on macOS when it differs from `name` (`ArtCraft` for `ArtCraft Studio`).
    pub mac_bundle_name: Option<&'static str>,
    pub kind: AppKind,
    /// Who publishes it, as the card's byline shows it (`ArtCraft`, `Tangency Labs`).
    pub publisher: &'static str,
    /// The publisher its Windows installer names, when it isn't ArtCraft's (`Learning Machines LLC`).
    pub msi_publisher: Option<&'static str>,
    /// The bundle id when it isn't `ai.storyteller.<id>` (a community app brings its own).
    pub bundle: Option<&'static str>,
    /// Older bundle ids its Flatpak shipped under (SolidCraft was `ai.storyteller.solidcraft`).
    pub former_bundles: &'static [&'static str],
    /// Its website, when it isn't a page on getartcraft.com (a community app has its own).
    pub site: Option<&'static str>,
    /// The Apple Developer team whose Developer ID signature its macOS build must carry. `None`
    /// while those builds carry no signature the launcher can check: it then opens a copy that's
    /// there and links to the releases, but installs nothing.
    pub mac_team: Option<&'static str>,
}

impl CraftApp {
    /// `ai.storyteller.<id>`: macOS bundle id, Flatpak id and Linux desktop-file id.
    pub fn bundle_id(&self) -> String {
        self.bundle.map_or_else(|| format!("ai.storyteller.{}", self.id), str::to_string)
    }

    pub fn releases_url(&self) -> String {
        format!("https://github.com/{}/releases", self.repo)
    }

    pub fn repo_url(&self) -> String {
        format!("https://github.com/{}", self.repo)
    }

    /// The app's website: its page on getartcraft.com, or a community app's own site.
    pub fn website(&self) -> String {
        self.site.map_or_else(|| self.repo_url(), str::to_string)
    }

    /// Whether the launcher installs this app from a `kind` package.
    pub fn installs(&self, kind: crate::PackageKind) -> bool {
        kind != crate::PackageKind::Dmg || self.mac_team.is_some()
    }

    /// Whether this app runs on the given operating system.
    pub fn runs_on(&self, os: crate::Os) -> bool {
        match os {
            crate::Os::FreeBsd => self.id != "artcraft",
            _ => true,
        }
    }

    /// Bundle ids this app may be installed under, current first.
    pub fn bundle_ids(&self) -> impl Iterator<Item = String> + '_ {
        std::iter::once(self.bundle_id()).chain(self.former_bundles.iter().map(|b| b.to_string()))
    }

    /// Asset stems this app's releases may use, current first.
    pub fn stems(&self) -> impl Iterator<Item = &'static str> + '_ {
        std::iter::once(self.id).chain(self.former_ids.iter().copied())
    }

    /// Display names this app's installers may have registered, current first.
    pub fn names(&self) -> impl Iterator<Item = &'static str> + '_ {
        std::iter::once(self.name).chain(self.former_names.iter().copied())
    }

    /// The primary `.app` bundle stem on macOS (`ArtCraft` or `PhotoCraft`).
    pub fn mac_bundle_name(&self) -> &'static str {
        self.mac_bundle_name.unwrap_or(self.name)
    }

    /// Bundle stems a macOS `.app` may carry (`ArtCraft` and `ArtCraft Studio`), release name first.
    pub fn mac_bundle_names(&self) -> impl Iterator<Item = &'static str> + '_ {
        let primary = self.mac_bundle_name();
        std::iter::once(primary)
            .chain(std::iter::once(self.name).filter(move |&n| n != primary))
            .chain(self.former_names.iter().copied().filter(move |&n| n != primary && n != self.name))
    }

    /// Whether this app's macOS build isn't Developer ID signed. Its download is still checked
    /// against the release's SHA-256 and its bundle id and version are checked after copying.
    pub fn mac_unsigned(&self) -> bool {
        self.mac_team == Some(UNSIGNED_MAC_TEAM)
    }

    pub fn by_id(id: &str) -> Option<&'static CraftApp> {
        crate::hosted_catalog::apps().iter().find(|a| a.id == id)
    }
}

const fn app(
    id: &'static str,
    name: &'static str,
    tagline: &'static str,
    [color, ink_light, ink_dark]: [[u8; 3]; 3],
    repo: &'static str,
) -> CraftApp {
    CraftApp {
        id,
        name,
        tagline,
        color,
        ink_light,
        ink_dark,
        repo,
        former_ids: &[],
        former_names: &[],
        mac_bundle_name: None,
        kind: AppKind::Official,
        publisher: "ArtCraft",
        msi_publisher: None,
        bundle: None,
        former_bundles: &[],
        site: None,
        mac_team: Some(ARTCRAFT_TEAM_ID),
    }
}

// The bundled suite, in website display order, then the approved community apps.
include!(concat!(env!("OUT_DIR"), "/catalog.rs"));

/// The Apple Developer team that signs every official Craft App and FlowApps (Learning
/// Machines LLC): the macOS side of the Windows publisher check.
/// Catalog `mac_team` value for an app whose Mac build carries no Developer ID signature.
pub const UNSIGNED_MAC_TEAM: &str = "unsigned";

pub const ARTCRAFT_TEAM_ID: &str = "DJ6XS33FX8";

/// FlowApps on the web: our GitHub.
pub const WEBSITE_URL: &str = "https://github.com/reachneoagent-commits";
/// The GitHub account every FlowApps repository lives under. The release feed can
/// only move an app within it; a community app installs from its own repo, pinned in [`SUITE`].
pub const GITHUB_OWNER: &str = "reachneoagent-commits";
/// FlowApps on GitHub: every app's source.
pub const GITHUB_URL: &str = "https://github.com/reachneoagent-commits";
/// FlowApps's social accounts: (service, URL). Shown as small icons.
pub const SOCIAL: [(&str, &str); 0] = [];

/// Where "community" links point: our GitHub (FlowApps has no Discord).
pub const DISCORD_URL: &str = "https://github.com/reachneoagent-commits";
pub const DISCORD_APP_URL: &str = "https://github.com/reachneoagent-commits";

/// FlowApps itself, described like a Craft App so it can update itself with the same
/// machinery (asset names `flowapps-launcher-<ver>-…`, bundle id `ai.reachneo.flowapps`,
/// `FlowApps.app`). Not part of [`SUITE`].
pub const LAUNCHER: CraftApp = CraftApp {
    mac_team: Some(UNSIGNED_MAC_TEAM),
    bundle: Some("ai.reachneo.flowapps"),
    ..app(
        "flowapps-launcher",
        "FlowApps",
        "Installs and updates your apps",
        [[0x0b, 0x0b, 0x0c], [0x0b, 0x0b, 0x0c], [0xf0, 0xee, 0xea]],
        "reachneoagent-commits/FlowApps-Launcher",
    )
};

/// The launcher's own repo, checked so it can say when a newer launcher is out.
pub const LAUNCHER_REPO: &str = "reachneoagent-commits/FlowApps-Launcher";
/// The launcher's own asset stem.
pub const LAUNCHER_ID: &str = "flowapps-launcher";

/// A Craft App's 128 px icon (PNG), as copied from its repo into `assets/app-icons`: the UI
/// draws it, and the Linux install puts it in the desktop menu.
pub fn app_icon_png(id: &str) -> Option<&'static [u8]> {
    if let Some(icon) = crate::hosted_catalog::icon(id) {
        return Some(icon);
    }
    Some(match id {
        "gramflow" => include_bytes!("../../../assets/app-icons/gramflow.png"),
        "voxflow" => include_bytes!("../../../assets/app-icons/voxflow.png"),
        "mailflow" => include_bytes!("../../../assets/app-icons/mailflow.png"),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_app_has_its_icon() {
        for app in &SUITE {
            let png = app_icon_png(app.id).unwrap_or_else(|| panic!("no icon for {}", app.id));
            assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"), "{} icon isn't a PNG", app.id);
        }
        assert!(app_icon_png("notacraft").is_none());
    }

    /// Every catalog entry must have a distinct lookup key.
    #[test]
    fn suite_has_three_apps_with_unique_ids() {
        assert_eq!(SUITE.len(), 3);
        let mut ids: Vec<_> = SUITE.iter().map(|a| a.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), SUITE.len());
        for a in &SUITE {
            assert_eq!(a.name.to_lowercase(), a.id);
            assert!(a.repo.starts_with(&format!("{GITHUB_OWNER}/")), "{} must live under {GITHUB_OWNER}", a.id);
            assert_eq!(a.website(), format!("https://github.com/{}", a.repo));
        }
    }

    #[test]
    fn apps_point_at_their_own_repos_and_bundle_ids() {
        let expected = [
            ("gramflow", "reachneoagent-commits/gramflow", "com.gramflow.app"),
            ("voxflow", "reachneoagent-commits/voxflow", "com.voxflow.app"),
            ("mailflow", "reachneoagent-commits/mailflow", "com.flowapps.mailflow"),
        ];
        for (id, repo, bundle) in expected {
            let app = CraftApp::by_id(id).unwrap();
            assert_eq!((app.repo, app.bundle_id().as_str()), (repo, bundle));
        }
    }

    #[test]
    fn mailflow_also_answers_to_mailcraft_release_names() {
        let mf = CraftApp::by_id("mailflow").unwrap();
        assert_eq!(mf.stems().collect::<Vec<_>>(), ["mailflow", "mailcraft"]);
    }

    #[test]
    fn the_launcher_lives_in_its_own_repo() {
        assert_eq!(LAUNCHER_REPO, "reachneoagent-commits/FlowApps-Launcher");
        assert_eq!(LAUNCHER.repo, LAUNCHER_REPO);
    }

    #[test]
    fn our_apps_install_on_mac_checked_by_checksum_not_signature() {
        for a in &SUITE {
            assert!(a.mac_unsigned() && a.installs(crate::PackageKind::Dmg), "{}", a.id);
        }
    }
}

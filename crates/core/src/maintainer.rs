//! Settings → Maintainer: keeping the FlowApps forks in step with the projects they came from.
//!
//! After signing in to GitHub with an account that can push to the FlowApps repositories, the
//! maintainer sees each app's upstream status and can run its **Upstream check**, **Upstream
//! prepare** and release workflows (see `docs/UPSTREAM.md` in each repository). Everyone else's
//! launcher shows nothing: GitHub refuses those calls without write access anyway.
//!
//! This module is pure and wasm-safe: the list of repositories, what the UI shows and the parsing
//! of GitHub's answers. The desktop backend does the HTTP and keeps the token in the system
//! keychain.

use serde_json::Value;

/// The GitHub organisation that hosts the FlowApps repositories.
pub const ORG: &str = "reachneoagent-commits";
/// Write access to this repository is what makes an account a maintainer.
pub const ACCESS_REPO: &str = "reachneoagent-commits/FlowApps-Launcher";
/// The GitHub OAuth App (or GitHub App) that the device-code sign-in uses. A client ID is not a
/// secret: it is the "FlowApps Launcher Maintainer" OAuth App of reachneoagent-commits, with
/// device flow on. A build can point at another app with `FLOWAPPS_LAUNCHER_GITHUB_CLIENT_ID`;
/// set it empty to leave the Maintainer section out.
pub const CLIENT_ID: &str = match option_env!("FLOWAPPS_LAUNCHER_GITHUB_CLIENT_ID") {
    Some(id) => id,
    None => "Ov23liRU7wiG9wvCthY9",
};
/// OAuth scope: running workflows and reading issues and pull requests of private repositories.
/// Ignored when [`CLIENT_ID`] is a GitHub App, whose own permissions apply instead.
pub const SCOPE: &str = "repo";
/// The only page the sign-in sends the person to.
pub const DEVICE_PAGE: &str = "https://github.com/login/device";
/// The prefix of the branches `upstream.py prepare` pushes.
pub const SYNC_BRANCH_PREFIX: &str = "upstream-sync/";

/// How an app is released.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Release {
    /// Run this workflow; it publishes the version in the repository.
    Workflow(&'static str),
    /// Run this workflow with the version to publish in the named input.
    WorkflowWithVersion { file: &'static str, input: &'static str },
    /// Pushing a `v*` tag releases: the launcher opens GitHub's new-release page.
    Tag,
}

/// One maintained repository.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Repo {
    pub name: &'static str,
    /// `owner/name`.
    pub repo: &'static str,
    /// The weekly upstream check workflow, where the app has an upstream.
    pub check: Option<&'static str>,
    /// The on-demand prepare workflow.
    pub prepare: Option<&'static str>,
    pub release: Option<Release>,
}

impl Repo {
    pub fn web(&self) -> String {
        format!("https://github.com/{}", self.repo)
    }
}

const CHECK: Option<&str> = Some("upstream-check.yml");
const PREPARE: Option<&str> = Some("upstream-prepare.yml");

/// Every repository the Maintainer section lists, the launcher first.
pub const REPOS: &[Repo] = &[
    Repo { name: "FlowApps Launcher", repo: ACCESS_REPO, check: CHECK, prepare: PREPARE, release: Some(Release::Workflow("release.yml")) },
    Repo {
        name: "Gramflow",
        repo: "reachneoagent-commits/gramflow",
        check: Some("upstream-watch.yml"),
        prepare: Some("upstream-merge.yml"),
        release: Some(Release::Workflow("gramflow-release.yml")),
    },
    Repo { name: "Voxflow", repo: "reachneoagent-commits/voxflow", check: CHECK, prepare: PREPARE, release: Some(Release::Workflow("release.yml")) },
    Repo { name: "Mailflow", repo: "reachneoagent-commits/mailflow", check: CHECK, prepare: PREPARE, release: Some(Release::Workflow("release.yml")) },
    Repo { name: "Buildflow", repo: "reachneoagent-commits/buildflow", check: CHECK, prepare: PREPARE, release: Some(Release::Workflow("release.yml")) },
    Repo {
        name: "Writeflow",
        repo: "reachneoagent-commits/writeflow",
        check: CHECK,
        prepare: PREPARE,
        release: Some(Release::WorkflowWithVersion { file: "release-app.yml", input: "version" }),
    },
    Repo {
        name: "VPNFlow",
        repo: "reachneoagent-commits/vpnflow",
        check: CHECK,
        prepare: PREPARE,
        release: Some(Release::WorkflowWithVersion { file: "release-desktop.yml", input: "confirm_version" }),
    },
    Repo {
        name: "Irisflow",
        repo: "reachneoagent-commits/irisflow",
        // Not a fork: the check reports newer LiveKit releases; updates come as Dependabot PRs.
        check: CHECK,
        prepare: None,
        release: Some(Release::WorkflowWithVersion { file: "release-app.yml", input: "version" }),
    },
    Repo { name: "Wordflow", repo: "reachneoagent-commits/wordflow", check: CHECK, prepare: PREPARE, release: Some(Release::Tag) },
    Repo { name: "Excelflow", repo: "reachneoagent-commits/excelflow", check: CHECK, prepare: PREPARE, release: Some(Release::Tag) },
    Repo { name: "PDFflow", repo: "reachneoagent-commits/pdfflow", check: CHECK, prepare: PREPARE, release: Some(Release::Tag) },
    Repo { name: "Slideflow", repo: "reachneoagent-commits/slideflow", check: CHECK, prepare: PREPARE, release: Some(Release::Tag) },
    Repo { name: "TVflow", repo: "reachneoagent-commits/tvflow", check: None, prepare: None, release: None },
];

pub fn find(repo: &str) -> Option<&'static Repo> {
    REPOS.iter().find(|r| r.repo == repo)
}

/// The GitHub account the launcher is signed in with.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum Account {
    /// No sign-in in this build or on this system (no client ID, no keychain).
    #[default]
    Unavailable,
    SignedOut,
    /// Asking GitHub for a code.
    Starting,
    /// Waiting for the person to enter `user_code` at [`DEVICE_PAGE`].
    Waiting { user_code: String, expires_at: i64 },
    /// Checking the saved or new token.
    Verifying,
    SignedIn { login: String, can_write: bool },
}

/// A workflow run, as shown next to an app.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
    /// `queued`, `in_progress`, `completed`…
    pub status: String,
    /// `success`, `failure`, `cancelled`… once completed.
    pub conclusion: Option<String>,
    pub url: String,
    /// RFC 3339.
    pub created_at: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunState {
    Running,
    Succeeded,
    Failed,
}

impl Run {
    pub fn state(&self) -> RunState {
        if self.status != "completed" {
            RunState::Running
        } else if matches!(self.conclusion.as_deref(), Some("success" | "skipped" | "neutral")) {
            RunState::Succeeded
        } else {
            RunState::Failed
        }
    }
}

/// A titled GitHub page: the open upstream-update issue, the open sync pull request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    pub title: String,
    pub url: String,
}

/// What the Maintainer section knows about one repository.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RepoState {
    pub loaded: bool,
    pub loading: bool,
    pub default_branch: Option<String>,
    /// The open `upstream-update` issue: upstream has moved.
    pub issue: Option<Link>,
    /// The open pull request from an `upstream-sync/…` branch.
    pub sync_pr: Option<Link>,
    pub check_run: Option<Run>,
    pub prepare_run: Option<Run>,
    pub release_run: Option<Run>,
    pub error: Option<String>,
}

/// Everything the Maintainer section draws.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct View {
    pub account: Account,
    /// In the order of [`REPOS`].
    pub repos: Vec<RepoState>,
    /// The last thing that happened, and whether it was a failure.
    pub message: Option<(String, bool)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    SignIn,
    CancelSignIn,
    SignOut,
    /// Reload every repository's status.
    Refresh,
    /// Run one repository's upstream check.
    Check(&'static str),
    /// Run every repository's upstream check.
    CheckAll,
    /// Run one repository's upstream prepare.
    Prepare(&'static str),
    /// Run one repository's release workflow, with the version where it asks for one.
    Release { repo: &'static str, version: Option<String> },
}

/// The Maintainer service a desktop host offers. Quick to call: work happens on other threads.
pub trait Maintainer {
    /// Starts loading the saved sign-in the first time it's asked.
    fn view(&mut self) -> View;
    fn act(&mut self, action: Action);
}

// ---- GitHub's answers. All of it is untrusted. ----

/// GitHub's answer to the device-code request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceCode {
    pub device_code: String,
    pub user_code: String,
    pub interval: u64,
    pub expires_in: i64,
}

fn str_field(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(Value::as_str).map(str::to_string)
}

/// A short, printable piece of untrusted text.
fn clean(s: &str, max: usize) -> String {
    s.chars().filter(|c| !c.is_control()).take(max).collect()
}

/// Only GitHub's own web pages are kept as links.
fn github_page(url: Option<String>) -> Option<String> {
    url.filter(|u| u.starts_with("https://github.com/") && crate::release::safe_web_link(u))
}

pub fn parse_device_code(text: &str) -> Result<DeviceCode, String> {
    let v: Value = serde_json::from_str(text).map_err(|e| format!("unreadable answer from GitHub ({e})"))?;
    if let Some(error) = str_field(&v, "error") {
        return Err(clean(&str_field(&v, "error_description").unwrap_or(error), 200));
    }
    let device_code = str_field(&v, "device_code").filter(|c| !c.is_empty() && c.len() <= 200).ok_or("GitHub sent no device code")?;
    let user_code = str_field(&v, "user_code")
        .filter(|c| !c.is_empty() && c.len() <= 20 && c.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'))
        .ok_or("GitHub sent no sign-in code")?;
    let interval = v.get("interval").and_then(Value::as_u64).unwrap_or(5).clamp(1, 60);
    let expires_in = v.get("expires_in").and_then(Value::as_i64).unwrap_or(900).clamp(60, 3600);
    Ok(DeviceCode { device_code, user_code, interval, expires_in })
}

/// One poll of the device-code token endpoint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenPoll {
    Token(String),
    Pending,
    SlowDown,
    Expired,
    Denied,
    Failed(String),
}

pub fn parse_token_poll(text: &str) -> TokenPoll {
    let Ok(v) = serde_json::from_str::<Value>(text) else { return TokenPoll::Failed("unreadable answer from GitHub".into()) };
    if let Some(token) = str_field(&v, "access_token").filter(|t| !t.is_empty() && t.len() <= 500 && t.chars().all(|c| c.is_ascii_graphic())) {
        return TokenPoll::Token(token);
    }
    match str_field(&v, "error").as_deref() {
        Some("authorization_pending") => TokenPoll::Pending,
        Some("slow_down") => TokenPoll::SlowDown,
        Some("expired_token") => TokenPoll::Expired,
        Some("access_denied") => TokenPoll::Denied,
        Some(other) => TokenPoll::Failed(clean(&str_field(&v, "error_description").unwrap_or_else(|| other.to_string()), 200)),
        None => TokenPoll::Failed("GitHub sent no token".into()),
    }
}

/// The login from `GET /user`.
pub fn parse_login(text: &str) -> Option<String> {
    let v: Value = serde_json::from_str(text).ok()?;
    str_field(&v, "login").filter(|l| !l.is_empty() && l.len() <= 39 && l.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'))
}

/// From `GET /repos/{repo}`: whether this account can push, and the default branch.
pub fn parse_repo(text: &str) -> (bool, Option<String>) {
    let Ok(v) = serde_json::from_str::<Value>(text) else { return (false, None) };
    let push = v.pointer("/permissions/push").and_then(Value::as_bool).unwrap_or(false);
    let branch = str_field(&v, "default_branch").filter(|b| !b.is_empty() && b.len() <= 100 && !b.chars().any(|c| c.is_control() || c == ' '));
    (push, branch)
}

/// The newest run from `GET /repos/{repo}/actions/workflows/{file}/runs`.
pub fn parse_latest_run(text: &str) -> Option<Run> {
    let v: Value = serde_json::from_str(text).ok()?;
    let run = v.get("workflow_runs")?.as_array()?.first()?;
    Some(Run {
        status: clean(&str_field(run, "status")?, 30),
        conclusion: str_field(run, "conclusion").map(|c| clean(&c, 30)),
        url: github_page(str_field(run, "html_url"))?,
        created_at: clean(&str_field(run, "created_at").unwrap_or_default(), 40),
    })
}

/// The first open issue (not pull request) from `GET /repos/{repo}/issues`.
pub fn parse_issue(text: &str) -> Option<Link> {
    let v: Value = serde_json::from_str(text).ok()?;
    v.as_array()?.iter().filter(|i| i.get("pull_request").is_none()).find_map(|i| {
        Some(Link { title: clean(&str_field(i, "title")?, 200), url: github_page(str_field(i, "html_url"))? })
    })
}

/// The open pull request from an `upstream-sync/…` branch in `GET /repos/{repo}/pulls`.
pub fn parse_sync_pr(text: &str) -> Option<Link> {
    let v: Value = serde_json::from_str(text).ok()?;
    v.as_array()?.iter().find_map(|p| {
        let head = p.pointer("/head/ref").and_then(Value::as_str)?;
        if !head.starts_with(SYNC_BRANCH_PREFIX) {
            return None;
        }
        Some(Link { title: clean(&str_field(p, "title")?, 200), url: github_page(str_field(p, "html_url"))? })
    })
}

/// The body of a workflow dispatch.
pub fn dispatch_body(branch: &str, inputs: &[(&str, &str)]) -> String {
    let inputs: serde_json::Map<String, Value> = inputs.iter().map(|(k, v)| ((*k).to_string(), Value::String((*v).to_string()))).collect();
    serde_json::json!({ "ref": branch, "inputs": inputs }).to_string()
}

/// A version someone typed for a release: digits and dots, optionally a `-pre` part.
pub fn valid_version(v: &str) -> bool {
    let v = v.trim();
    !v.is_empty()
        && v.len() <= 40
        && v.chars().next().is_some_and(|c| c.is_ascii_digit())
        && v.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_repo_is_in_the_org_and_listed_once() {
        for (i, r) in REPOS.iter().enumerate() {
            assert!(r.repo.starts_with(&format!("{ORG}/")), "{}", r.repo);
            assert!(REPOS[i + 1..].iter().all(|o| o.repo != r.repo), "{} twice", r.repo);
            assert!(r.prepare.is_none() || r.check.is_some(), "{} prepares without a check", r.repo);
        }
        assert_eq!(REPOS[0].repo, ACCESS_REPO);
    }

    #[test]
    fn device_code_is_read_and_bounded() {
        let code = parse_device_code(
            r#"{"device_code":"abc","user_code":"WDJB-MJHT","verification_uri":"https://github.com/login/device","expires_in":900,"interval":5}"#,
        )
        .unwrap();
        assert_eq!(code, DeviceCode { device_code: "abc".into(), user_code: "WDJB-MJHT".into(), interval: 5, expires_in: 900 });
        assert!(parse_device_code(r#"{"device_code":"abc","user_code":"<script>"}"#).is_err());
        assert_eq!(parse_device_code(r#"{"error":"unauthorized_client","error_description":"Device flow is off"}"#).unwrap_err(), "Device flow is off");
    }

    #[test]
    fn token_polls_are_told_apart() {
        assert_eq!(parse_token_poll(r#"{"access_token":"gho_x","token_type":"bearer"}"#), TokenPoll::Token("gho_x".into()));
        assert_eq!(parse_token_poll(r#"{"error":"authorization_pending"}"#), TokenPoll::Pending);
        assert_eq!(parse_token_poll(r#"{"error":"slow_down","interval":10}"#), TokenPoll::SlowDown);
        assert_eq!(parse_token_poll(r#"{"error":"expired_token"}"#), TokenPoll::Expired);
        assert_eq!(parse_token_poll(r#"{"error":"access_denied"}"#), TokenPoll::Denied);
        assert!(matches!(parse_token_poll("nope"), TokenPoll::Failed(_)));
        assert!(matches!(parse_token_poll(r#"{"access_token":"has space"}"#), TokenPoll::Failed(_)));
    }

    #[test]
    fn repo_permission_and_branch() {
        assert_eq!(parse_repo(r#"{"default_branch":"main","permissions":{"admin":false,"push":true}}"#), (true, Some("main".into())));
        assert_eq!(parse_repo(r#"{"default_branch":"main","permissions":{"push":false}}"#), (false, Some("main".into())));
        assert_eq!(parse_repo(r#"{"default_branch":"main"}"#), (false, Some("main".into())));
        assert_eq!(parse_login(r#"{"login":"octo-cat"}"#), Some("octo-cat".into()));
        assert_eq!(parse_login(r#"{"login":"no way"}"#), None);
    }

    #[test]
    fn runs_issues_and_pulls_keep_only_github_pages() {
        let run = parse_latest_run(
            r#"{"workflow_runs":[{"status":"completed","conclusion":"failure","html_url":"https://github.com/o/r/actions/runs/1","created_at":"2026-11-27T10:00:00Z"}]}"#,
        )
        .unwrap();
        assert_eq!(run.state(), RunState::Failed);
        assert!(parse_latest_run(r#"{"workflow_runs":[]}"#).is_none());
        assert!(parse_latest_run(r#"{"workflow_runs":[{"status":"queued","html_url":"https://evil.example/x"}]}"#).is_none());
        let queued = parse_latest_run(r#"{"workflow_runs":[{"status":"queued","html_url":"https://github.com/o/r/actions/runs/2"}]}"#).unwrap();
        assert_eq!(queued.state(), RunState::Running);

        let issues = r#"[{"title":"PR","html_url":"https://github.com/o/r/pull/3","pull_request":{}},{"title":"Upstream moved","html_url":"https://github.com/o/r/issues/4"}]"#;
        assert_eq!(parse_issue(issues), Some(Link { title: "Upstream moved".into(), url: "https://github.com/o/r/issues/4".into() }));
        assert_eq!(parse_issue("[]"), None);

        let pulls = r#"[{"title":"other","html_url":"https://github.com/o/r/pull/1","head":{"ref":"feature"}},
                        {"title":"chore: merge upstream","html_url":"https://github.com/o/r/pull/2","head":{"ref":"upstream-sync/2026-11-27"}}]"#;
        assert_eq!(parse_sync_pr(pulls).unwrap().url, "https://github.com/o/r/pull/2");
    }

    #[test]
    fn dispatch_bodies_and_versions() {
        assert_eq!(dispatch_body("main", &[]), r#"{"inputs":{},"ref":"main"}"#);
        assert_eq!(dispatch_body("main", &[("version", "1.2.3")]), r#"{"inputs":{"version":"1.2.3"},"ref":"main"}"#);
        assert!(valid_version("0.3.2") && valid_version("1.0.0-rc.1"));
        assert!(!valid_version("") && !valid_version("v1") && !valid_version("1 2") && !valid_version("1;rm"));
    }
}

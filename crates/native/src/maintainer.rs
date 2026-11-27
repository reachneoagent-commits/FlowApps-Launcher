//! Settings → Maintainer, on the desktop: GitHub's device-code sign-in, the token in the system
//! keychain, and the GitHub API calls behind each button. See [`flowapps_launcher_core::maintainer`].
//!
//! Every call runs on its own thread; the UI reads [`View`] snapshots. The token lives only in
//! memory and the keychain, and is only ever sent to github.com and api.github.com. It is never
//! logged.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use flowapps_launcher_core::maintainer::{
    self as m, ACCESS_REPO, Account, Action, CLIENT_ID, REPOS, Release, RepoState, SCOPE, TokenPoll, View,
};

const API: &str = "https://api.github.com";
const USER_AGENT: &str = concat!("FlowAppsLauncher/", env!("CARGO_PKG_VERSION"));
const MAX_ANSWER: u64 = 2 << 20;
/// How long after starting a workflow its repository is looked at again, so the new run shows.
const SETTLE: Duration = Duration::from_secs(4);

/// The system keychain entry that holds the token.
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod keychain {
    const SERVICE: &str = "ai.reachneo.flowapps.github";
    const USER: &str = "maintainer";

    pub const AVAILABLE: bool = true;

    fn entry() -> Result<keyring::Entry, String> {
        keyring::Entry::new(SERVICE, USER).map_err(|e| e.to_string())
    }

    pub fn load() -> Result<Option<String>, String> {
        match entry()?.get_password() {
            Ok(token) => Ok(Some(token)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn save(token: &str) -> Result<(), String> {
        entry()?.set_password(token).map_err(|e| e.to_string())
    }

    pub fn delete() -> Result<(), String> {
        match entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod keychain {
    pub const AVAILABLE: bool = false;

    pub fn load() -> Result<Option<String>, String> {
        Ok(None)
    }

    pub fn save(_token: &str) -> Result<(), String> {
        Err("no keychain on this system".into())
    }

    pub fn delete() -> Result<(), String> {
        Ok(())
    }
}

#[derive(Default)]
struct State {
    view: View,
    token: Option<String>,
    /// Bumped by every sign-in, cancel and sign-out, so a stale worker stops.
    session: u64,
}

struct Shared {
    state: Mutex<State>,
    agent: ureq::Agent,
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn say(&self, text: impl Into<String>, failed: bool) {
        self.lock().view.message = Some((text.into(), failed));
    }

    /// The token, if `session` is still the current one.
    fn token(&self, session: u64) -> Option<String> {
        let s = self.lock();
        if s.session == session { s.token.clone() } else { None }
    }
}

/// The desktop's Maintainer service.
pub struct NativeMaintainer {
    shared: Arc<Shared>,
    started: bool,
}

impl NativeMaintainer {
    pub fn new() -> Self {
        let tls = ureq::tls::TlsConfig::builder().root_certs(ureq::tls::RootCerts::PlatformVerifier).build();
        let agent = ureq::Agent::config_builder()
            .tls_config(tls)
            .proxy(crate::proxy::pick())
            .timeout_global(Some(Duration::from_secs(20)))
            .http_status_as_error(false)
            // The token goes in a header: never follow a redirect off GitHub with it.
            .max_redirects(0)
            .max_redirects_will_error(false)
            .user_agent(USER_AGENT)
            .build()
            .new_agent();
        let view = View { repos: vec![RepoState::default(); REPOS.len()], ..View::default() };
        Self { shared: Arc::new(Shared { state: Mutex::new(State { view, ..State::default() }), agent }), started: false }
    }

    /// First look: read the saved sign-in. Not at startup, so the keychain is only touched once
    /// someone opens Settings.
    fn start(&mut self) {
        self.started = true;
        if CLIENT_ID.is_empty() || !keychain::AVAILABLE {
            return;
        }
        self.shared.lock().view.account = Account::Verifying;
        let shared = self.shared.clone();
        spawn(move || match keychain::load() {
            Ok(Some(token)) => {
                let session = {
                    let mut s = shared.lock();
                    s.token = Some(token);
                    s.session
                };
                verify(&shared, session);
            }
            Ok(None) => shared.lock().view.account = Account::SignedOut,
            Err(e) => {
                log::warn!("maintainer: keychain: {e}");
                let mut s = shared.lock();
                s.view.account = Account::SignedOut;
                s.view.message = Some((format!("Couldn't read the keychain ({e})."), true));
            }
        });
    }
}

impl Default for NativeMaintainer {
    fn default() -> Self {
        Self::new()
    }
}

impl m::Maintainer for NativeMaintainer {
    fn view(&mut self) -> View {
        if !self.started {
            self.start();
        }
        self.shared.lock().view.clone()
    }

    fn act(&mut self, action: Action) {
        let shared = self.shared.clone();
        match action {
            Action::SignIn => {
                let session = {
                    let mut s = shared.lock();
                    if !matches!(s.view.account, Account::SignedOut) {
                        return;
                    }
                    s.session += 1;
                    s.view.account = Account::Starting;
                    s.view.message = None;
                    s.session
                };
                spawn(move || sign_in(&shared, session));
            }
            Action::CancelSignIn => {
                let mut s = shared.lock();
                if matches!(s.view.account, Account::Starting | Account::Waiting { .. }) {
                    s.session += 1;
                    s.view.account = Account::SignedOut;
                }
            }
            Action::SignOut => {
                {
                    let mut s = shared.lock();
                    s.session += 1;
                    s.token = None;
                    s.view = View { account: Account::SignedOut, repos: vec![RepoState::default(); REPOS.len()], message: None };
                }
                spawn(move || {
                    if let Err(e) = keychain::delete() {
                        shared.say(format!("Signed out, but the keychain entry stayed ({e})."), true);
                    }
                });
            }
            Action::Refresh => {
                let session = shared.lock().session;
                refresh_all(&shared, session);
            }
            Action::Check(repo) => dispatch_named(&shared, repo, Workflow::Check, None),
            Action::CheckAll => {
                for r in REPOS.iter().filter(|r| r.check.is_some()) {
                    dispatch_named(&shared, r.repo, Workflow::Check, None);
                }
            }
            Action::Prepare(repo) => dispatch_named(&shared, repo, Workflow::Prepare, None),
            Action::Release { repo, version } => dispatch_named(&shared, repo, Workflow::Release, version),
        }
    }
}

fn spawn(f: impl FnOnce() + Send + 'static) {
    if let Err(e) = std::thread::Builder::new().name("maintainer".into()).spawn(f) {
        log::warn!("maintainer: couldn't start a thread: {e}");
    }
}

fn read(resp: &mut ureq::http::Response<ureq::Body>) -> Result<String, String> {
    crate::http::read_text_capped(resp, MAX_ANSWER)
}

/// POSTs a form to github.com (the device-code endpoints) and reads the JSON answer.
fn post_form(shared: &Shared, url: &str, form: &[(&str, &str)]) -> Result<String, String> {
    let mut resp = shared
        .agent
        .post(url)
        .header("Accept", "application/json")
        .send_form(form.iter().copied())
        .map_err(|e| format!("couldn't reach GitHub ({e})"))?;
    read(&mut resp)
}

/// A GitHub API answer: status and body.
fn api(shared: &Shared, token: &str, method: &str, path: &str, body: Option<String>) -> Result<(u16, String), String> {
    let url = format!("{API}{path}");
    let auth = format!("Bearer {token}");
    let resp = match (method, body) {
        ("POST", Some(body)) => shared
            .agent
            .post(&url)
            .header("Authorization", &auth)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .content_type("application/json")
            .send(body),
        _ => shared
            .agent
            .get(&url)
            .header("Authorization", &auth)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .call(),
    };
    let mut resp = resp.map_err(|e| format!("couldn't reach GitHub ({e})"))?;
    let status = resp.status().as_u16();
    Ok((status, read(&mut resp)?))
}

/// GitHub's own explanation in an error answer, short.
fn github_message(status: u16, body: &str) -> String {
    let said = serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v.get("message").and_then(|m| m.as_str()).map(|m| m.chars().filter(|c| !c.is_control()).take(200).collect::<String>()));
    match said {
        Some(said) if !said.is_empty() => format!("GitHub said {status}: {said}"),
        _ => format!("GitHub said {status}"),
    }
}

/// A token GitHub no longer accepts: forget it.
fn drop_token(shared: &Shared, session: u64) {
    {
        let mut s = shared.lock();
        if s.session != session {
            return;
        }
        s.session += 1;
        s.token = None;
        s.view = View {
            account: Account::SignedOut,
            repos: vec![RepoState::default(); REPOS.len()],
            message: Some(("GitHub no longer accepts this sign-in. Sign in again.".into(), true)),
        };
    }
    if let Err(e) = keychain::delete() {
        log::warn!("maintainer: keychain: {e}");
    }
}

fn sign_in(shared: &Arc<Shared>, session: u64) {
    let current = |shared: &Shared| shared.lock().session == session;
    let code = match post_form(shared, "https://github.com/login/device/code", &[("client_id", CLIENT_ID), ("scope", SCOPE)])
        .and_then(|text| m::parse_device_code(&text))
    {
        Ok(code) => code,
        Err(e) => {
            let mut s = shared.lock();
            if s.session == session {
                s.view.account = Account::SignedOut;
                s.view.message = Some((format!("Couldn't start the sign-in: {e}"), true));
            }
            return;
        }
    };
    let expires_at = crate::now() + code.expires_in;
    {
        let mut s = shared.lock();
        if s.session != session {
            return;
        }
        s.view.account = Account::Waiting { user_code: code.user_code.clone(), expires_at };
    }
    let mut interval = code.interval;
    let grant = "urn:ietf:params:oauth:grant-type:device_code";
    loop {
        // Sleep in short steps so Cancel is quick to take.
        for _ in 0..interval {
            std::thread::sleep(Duration::from_secs(1));
            if !current(shared) {
                return;
            }
        }
        if crate::now() > expires_at {
            finish_sign_in(shared, session, Err("The code expired. Sign in again.".into()));
            return;
        }
        let poll = match post_form(
            shared,
            "https://github.com/login/oauth/access_token",
            &[("client_id", CLIENT_ID), ("device_code", &code.device_code), ("grant_type", grant)],
        ) {
            Ok(text) => m::parse_token_poll(&text),
            // A dropped connection isn't the end: try again at the next interval.
            Err(e) => {
                log::warn!("maintainer: sign-in poll: {e}");
                TokenPoll::Pending
            }
        };
        match poll {
            TokenPoll::Pending => {}
            TokenPoll::SlowDown => interval = (interval + 5).min(60),
            TokenPoll::Token(token) => {
                finish_sign_in(shared, session, Ok(token));
                return;
            }
            TokenPoll::Expired => return finish_sign_in(shared, session, Err("The code expired. Sign in again.".into())),
            TokenPoll::Denied => return finish_sign_in(shared, session, Err("The sign-in was cancelled on GitHub.".into())),
            TokenPoll::Failed(e) => return finish_sign_in(shared, session, Err(format!("The sign-in failed: {e}"))),
        }
    }
}

fn finish_sign_in(shared: &Arc<Shared>, session: u64, outcome: Result<String, String>) {
    match outcome {
        Ok(token) => {
            {
                let mut s = shared.lock();
                if s.session != session {
                    return;
                }
                s.token = Some(token.clone());
                s.view.account = Account::Verifying;
            }
            if let Err(e) = keychain::save(&token) {
                shared.say(format!("Signed in for now; the keychain didn't keep it ({e})."), true);
            }
            verify(shared, session);
        }
        Err(e) => {
            let mut s = shared.lock();
            if s.session == session {
                s.view.account = Account::SignedOut;
                s.view.message = Some((e, true));
            }
        }
    }
}

/// Who the token belongs to, and whether it can push to the FlowApps repositories.
fn verify(shared: &Arc<Shared>, session: u64) {
    let Some(token) = shared.token(session) else { return };
    let user = api(shared, &token, "GET", "/user", None);
    let login = match user {
        Ok((401, _)) => return drop_token(shared, session),
        Ok((200, body)) => m::parse_login(&body),
        Ok((status, body)) => {
            set_unverified(shared, session, github_message(status, &body));
            return;
        }
        Err(e) => {
            set_unverified(shared, session, e);
            return;
        }
    };
    let Some(login) = login else {
        set_unverified(shared, session, "GitHub didn't say who this is".into());
        return;
    };
    // 404 here means the account can't even see the repository.
    let can_write = match api(shared, &token, "GET", &format!("/repos/{ACCESS_REPO}"), None) {
        Ok((401, _)) => return drop_token(shared, session),
        Ok((200, body)) => m::parse_repo(&body).0,
        Ok(_) | Err(_) => false,
    };
    {
        let mut s = shared.lock();
        if s.session != session {
            return;
        }
        s.view.account = Account::SignedIn { login, can_write };
    }
    if can_write {
        refresh_all(shared, session);
    }
}

/// Couldn't reach GitHub: keep the token, offer sign-out and a retry through Refresh.
fn set_unverified(shared: &Shared, session: u64, why: String) {
    let mut s = shared.lock();
    if s.session == session {
        s.view.account = Account::SignedIn { login: String::new(), can_write: false };
        s.view.message = Some((format!("Couldn't check the sign-in: {why}"), true));
    }
}

fn refresh_all(shared: &Arc<Shared>, session: u64) {
    {
        let mut s = shared.lock();
        if s.session != session || s.token.is_none() {
            return;
        }
        // Not yet verified (GitHub was unreachable): verify again instead.
        if matches!(&s.view.account, Account::SignedIn { login, .. } if login.is_empty()) {
            s.view.account = Account::Verifying;
            s.view.message = None;
            drop(s);
            let shared = shared.clone();
            spawn(move || verify(&shared, session));
            return;
        }
        if !matches!(s.view.account, Account::SignedIn { can_write: true, .. }) {
            return;
        }
        s.view.message = None;
    }
    for i in 0..REPOS.len() {
        refresh_one(shared, session, i, Duration::ZERO);
    }
}

fn refresh_one(shared: &Arc<Shared>, session: u64, index: usize, after: Duration) {
    {
        let mut s = shared.lock();
        let Some(state) = s.view.repos.get_mut(index) else { return };
        state.loading = true;
    }
    let shared = shared.clone();
    spawn(move || {
        std::thread::sleep(after);
        let Some(token) = shared.token(session) else { return };
        let Some(repo) = REPOS.get(index) else { return };
        let fresh = load_repo(&shared, &token, repo);
        let mut s = shared.lock();
        if s.session != session {
            return;
        }
        if let Some(state) = s.view.repos.get_mut(index) {
            *state = fresh;
        }
    });
}

fn load_repo(shared: &Shared, token: &str, repo: &m::Repo) -> RepoState {
    let r = repo.repo;
    let mut state = RepoState { loaded: true, ..RepoState::default() };
    match api(shared, token, "GET", &format!("/repos/{r}"), None) {
        Ok((200, body)) => state.default_branch = m::parse_repo(&body).1,
        Ok((status, body)) => {
            state.error = Some(github_message(status, &body));
            return state;
        }
        Err(e) => {
            state.error = Some(e);
            return state;
        }
    }
    let get = |path: String| match api(shared, token, "GET", &path, None) {
        Ok((200, body)) => Some(body),
        _ => None,
    };
    if repo.check.is_some() {
        state.issue = get(format!("/repos/{r}/issues?labels=upstream-update&state=open&per_page=5")).and_then(|b| m::parse_issue(&b));
    }
    if repo.prepare.is_some() {
        state.sync_pr = get(format!("/repos/{r}/pulls?state=open&per_page=50")).and_then(|b| m::parse_sync_pr(&b));
    }
    let latest = |file: Option<&str>| file.and_then(|f| get(format!("/repos/{r}/actions/workflows/{f}/runs?per_page=1"))).and_then(|b| m::parse_latest_run(&b));
    state.check_run = latest(repo.check);
    state.prepare_run = latest(repo.prepare);
    state.release_run = latest(match repo.release {
        Some(Release::Workflow(file) | Release::WorkflowWithVersion { file, .. }) => Some(file),
        Some(Release::Tag) | None => None,
    });
    state
}

#[derive(Clone, Copy)]
enum Workflow {
    Check,
    Prepare,
    Release,
}

/// Runs one of a repository's workflows on its default branch.
fn dispatch_named(shared: &Arc<Shared>, repo: &'static str, which: Workflow, version: Option<String>) {
    let Some(index) = REPOS.iter().position(|r| r.repo == repo) else { return };
    let Some(r) = REPOS.get(index) else { return };
    let (file, input, label) = match which {
        Workflow::Check => (r.check, None, "Upstream check"),
        Workflow::Prepare => (r.prepare, None, "Upstream prepare"),
        Workflow::Release => match r.release {
            Some(Release::Workflow(file)) => (Some(file), None, "Release"),
            Some(Release::WorkflowWithVersion { file, input }) => (Some(file), Some(input), "Release"),
            Some(Release::Tag) | None => (None, None, "Release"),
        },
    };
    let Some(file) = file else { return };
    let version = match (input, version) {
        (Some(_), Some(v)) if m::valid_version(&v) => Some(v.trim().to_string()),
        (Some(_), _) => {
            shared.say(format!("{}: enter a version like 1.2.3 to release.", r.name), true);
            return;
        }
        (None, _) => None,
    };
    let (session, branch) = {
        let s = shared.lock();
        if s.token.is_none() || !matches!(s.view.account, Account::SignedIn { can_write: true, .. }) {
            return;
        }
        (s.session, s.view.repos.get(index).and_then(|st| st.default_branch.clone()))
    };
    let shared = shared.clone();
    spawn(move || {
        let Some(token) = shared.token(session) else { return };
        let branch = match branch {
            Some(b) => b,
            None => match api(&shared, &token, "GET", &format!("/repos/{repo}"), None) {
                Ok((200, body)) => m::parse_repo(&body).1.unwrap_or_else(|| "main".into()),
                Ok((status, body)) => return shared.say(format!("{} {label}: {}", r.name, github_message(status, &body)), true),
                Err(e) => return shared.say(format!("{} {label}: {e}", r.name), true),
            },
        };
        let inputs: Vec<(&str, &str)> = match (input, version.as_deref()) {
            (Some(name), Some(v)) => vec![(name, v)],
            _ => Vec::new(),
        };
        let body = m::dispatch_body(&branch, &inputs);
        match api(&shared, &token, "POST", &format!("/repos/{repo}/actions/workflows/{file}/dispatches"), Some(body)) {
            Ok((204 | 200, _)) => {
                shared.say(format!("Started {label} for {}.", r.name), false);
                refresh_one(&shared, session, index, SETTLE);
            }
            Ok((401, _)) => drop_token(&shared, session),
            Ok((status, body)) => shared.say(format!("{} {label}: {}", r.name, github_message(status, &body)), true),
            Err(e) => shared.say(format!("{} {label}: {e}", r.name), true),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use flowapps_launcher_core::maintainer::Maintainer;

    #[test]
    fn without_a_client_id_or_keychain_it_stays_unavailable_and_inert() {
        let mut maintainer = NativeMaintainer::new();
        let view = maintainer.view();
        if CLIENT_ID.is_empty() || !keychain::AVAILABLE {
            assert_eq!(view.account, Account::Unavailable);
            maintainer.act(Action::SignIn);
            maintainer.act(Action::CheckAll);
            assert_eq!(maintainer.view().account, Account::Unavailable);
        }
        assert_eq!(view.repos.len(), REPOS.len());
    }

    #[test]
    fn github_errors_are_short_and_plain() {
        assert_eq!(github_message(404, r#"{"message":"Not Found"}"#), "GitHub said 404: Not Found");
        assert_eq!(github_message(500, "<html>"), "GitHub said 500");
    }
}

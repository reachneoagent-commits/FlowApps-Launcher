//! Settings → Maintainer: sign in to GitHub, then see and run each FlowApps repository's upstream
//! and release workflows. Nothing runs or merges by itself: every workflow starts from a button,
//! and a release asks first.
//!
//! Only the FlowApps maintainers ever see more than the sign-in button, so this panel is English
//! only: its strings are plain literals, not `tl!` (the catalogs don't carry them).

use egui::{RichText, Ui};
use flowapps_launcher_core::maintainer::{Account, Action, DEVICE_PAGE, REPOS, Release, Repo, RepoState, Run, RunState, View};

use crate::app::Host;
use crate::sheet;
use crate::theme::Tokens;
use crate::widgets::{self, Kind};

/// The release waiting for confirmation: the repository and the version typed so far.
#[derive(Clone, Default)]
struct Confirm {
    repo: Option<&'static str>,
    version: String,
}

pub(crate) fn settings_maintainer(ui: &mut Ui, host: &mut dyn Host) {
    let Some(maintainer) = host.maintainer() else { return };
    let view = maintainer.view();
    if view.account == Account::Unavailable {
        return;
    }
    let mut actions = Vec::new();
    sheet::panel(ui, "Maintainer", |ui| draw(ui, &view, &mut actions));
    let busy = matches!(view.account, Account::Starting | Account::Waiting { .. } | Account::Verifying)
        || view.repos.iter().any(|r| r.loading || [&r.check_run, &r.prepare_run, &r.release_run].into_iter().flatten().any(|run| run.state() == RunState::Running));
    if busy {
        ui.ctx().request_repaint_after(std::time::Duration::from_secs(1));
    }
    for action in actions {
        maintainer.act(action);
    }
}

fn draw(ui: &mut Ui, view: &View, actions: &mut Vec<Action>) {
    let t = Tokens::get(ui.ctx());
    if let Some((text, failed)) = &view.message {
        ui.label(RichText::new(text).color(if *failed { t.bad } else { t.good }));
        ui.add_space(6.0);
    }
    match &view.account {
        Account::Unavailable => {}
        Account::SignedOut => {
            ui.label(RichText::new("For the people who maintain the FlowApps repositories.").color(t.muted));
            ui.add_space(6.0);
            if widgets::button(ui, "Maintainer sign-in", Kind::Secondary, 0.0).clicked() {
                actions.push(Action::SignIn);
            }
        }
        Account::Starting | Account::Verifying => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(RichText::new("Talking to GitHub…").color(t.muted));
            });
        }
        Account::Waiting { user_code, .. } => {
            ui.label(RichText::new("Enter this code on GitHub to sign in:").color(t.muted));
            ui.label(RichText::new(user_code).monospace().size(22.0).strong());
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                if widgets::button(ui, "Copy code and open GitHub", Kind::Primary, 0.0).clicked() {
                    ui.ctx().copy_text(user_code.clone());
                    widgets::open_external_link(ui.ctx(), DEVICE_PAGE);
                }
                if widgets::button(ui, "Cancel", Kind::Secondary, 0.0).clicked() {
                    actions.push(Action::CancelSignIn);
                }
            });
        }
        Account::SignedIn { login, can_write } => {
            if login.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    if widgets::button(ui, "Try again", Kind::Secondary, 0.0).clicked() {
                        actions.push(Action::Refresh);
                    }
                    if widgets::button(ui, "Sign out", Kind::Secondary, 0.0).clicked() {
                        actions.push(Action::SignOut);
                    }
                });
                return;
            }
            ui.label(RichText::new(format!("Signed in to GitHub as {login}.")).color(t.muted));
            if !can_write {
                ui.label(RichText::new("This account can't push to the FlowApps repositories, so there's nothing to run.").color(t.muted));
                ui.add_space(6.0);
                if widgets::button(ui, "Sign out", Kind::Secondary, 0.0).clicked() {
                    actions.push(Action::SignOut);
                }
                return;
            }
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                if widgets::button(ui, "Check all apps", Kind::Primary, 0.0).on_hover_text("Run every app's upstream check now").clicked() {
                    actions.push(Action::CheckAll);
                }
                if widgets::button(ui, "Refresh", Kind::Secondary, 0.0).clicked() {
                    actions.push(Action::Refresh);
                }
                if widgets::button(ui, "Sign out", Kind::Secondary, 0.0).clicked() {
                    actions.push(Action::SignOut);
                }
            });
            ui.add_space(4.0);
            for (repo, state) in REPOS.iter().zip(&view.repos) {
                ui.separator();
                repo_row(ui, repo, state, actions);
            }
        }
    }
}

fn run_line(ui: &mut Ui, label: &str, run: &Run) {
    let t = Tokens::get(ui.ctx());
    let (word, color) = match run.state() {
        RunState::Running => ("running", t.warn),
        RunState::Succeeded => ("passed", t.good),
        RunState::Failed => (run.conclusion.as_deref().unwrap_or("failed"), t.bad),
    };
    let date = run.created_at.get(..10).unwrap_or(&run.created_at);
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(format!("{label}: {word}")).color(color));
        ui.label(RichText::new(date).color(t.muted));
        widgets::hud_link(ui, "Open", &run.url);
    });
}

fn repo_row(ui: &mut Ui, repo: &'static Repo, state: &RepoState, actions: &mut Vec<Action>) {
    let t = Tokens::get(ui.ctx());
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(repo.name).strong());
        widgets::hud_link(ui, "GitHub", &repo.web());
        if state.loading {
            ui.spinner();
        }
    });
    if let Some(error) = &state.error {
        ui.label(RichText::new(error).color(t.bad));
        return;
    }
    if !state.loaded {
        return;
    }
    if repo.check.is_none() {
        ui.label(RichText::new("No upstream to follow.").color(t.muted));
    }
    match &state.issue {
        Some(issue) => {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Upstream has moved:").color(t.warn));
                widgets::hud_link(ui, &issue.title, &issue.url);
            });
        }
        None if repo.check.is_some() => {
            ui.label(RichText::new("Up to date with upstream, as of the last check.").color(t.muted));
        }
        None => {}
    }
    if let Some(pr) = &state.sync_pr {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Waiting for review:").color(t.warn));
            widgets::hud_link(ui, &pr.title, &pr.url);
        });
    }
    for (label, run) in [("Check", &state.check_run), ("Prepare", &state.prepare_run), ("Release", &state.release_run)] {
        if let Some(run) = run {
            run_line(ui, label, run);
        }
    }

    let confirm_id = egui::Id::unique("maintainer-release-confirm");
    let mut confirm: Confirm = ui.data(|d| d.get_temp(confirm_id)).unwrap_or_default();
    let confirming = confirm.repo == Some(repo.repo);
    ui.add_space(4.0);
    ui.horizontal_wrapped(|ui| {
        if repo.check.is_some() && widgets::button(ui, "Check", Kind::Secondary, 0.0).on_hover_text("Look for new upstream changes").clicked() {
            actions.push(Action::Check(repo.repo));
        }
        if repo.prepare.is_some()
            && widgets::button(ui, "Prepare update", Kind::Secondary, 0.0)
                .on_hover_text("Merge upstream on a branch and open a pull request for review")
                .clicked()
        {
            actions.push(Action::Prepare(repo.repo));
        }
        match repo.release {
            Some(Release::Tag) => {
                if widgets::button(ui, "Release", Kind::Secondary, 0.0).on_hover_text("Tag a v* release on GitHub").clicked() {
                    widgets::open_external_link(ui.ctx(), &format!("{}/releases/new", repo.web()));
                }
            }
            Some(_) if !confirming => {
                if widgets::button(ui, "Release", Kind::Secondary, 0.0).clicked() {
                    confirm = Confirm { repo: Some(repo.repo), version: String::new() };
                }
            }
            Some(_) | None => {}
        }
    });
    if confirming {
        let needs_version = matches!(repo.release, Some(Release::WorkflowWithVersion { .. }));
        ui.add_space(4.0);
        ui.label(RichText::new(format!("Publish a new {} release for everyone?", repo.name)).color(t.warn));
        if needs_version {
            ui.horizontal(|ui| {
                let label = ui.label("Version");
                ui.add(egui::TextEdit::singleline(&mut confirm.version).hint_text("1.2.3").desired_width(120.0)).labelled_by(label.id);
            });
        }
        let ready = !needs_version || flowapps_launcher_core::maintainer::valid_version(&confirm.version);
        ui.horizontal_wrapped(|ui| {
            if ui.add_enabled_ui(ready, |ui| widgets::button(ui, "Release now", Kind::Danger, 0.0)).inner.clicked() {
                let version = needs_version.then(|| confirm.version.trim().to_string());
                actions.push(Action::Release { repo: repo.repo, version });
                confirm = Confirm::default();
            }
            if widgets::button(ui, "Cancel", Kind::Secondary, 0.0).clicked() {
                confirm = Confirm::default();
            }
        });
    }
    ui.data_mut(|d| d.insert_temp(confirm_id, confirm));
}

//! Headless check of the launcher backend against the real GitHub releases.
//!
//!   cargo run -p flowapps-launcher-native --example smoke            # read-only: latest, detected, plan
//!   cargo run -p flowapps-launcher-native --example smoke -- install voxflow
//!   cargo run -p flowapps-launcher-native --example smoke -- uninstall voxflow
use flowapps_launcher_core::protocol::{Backend, Command, Event};
use flowapps_launcher_native::NativeBackend;
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut backend = NativeBackend::new(|| {});
    backend.send(Command::CheckForUpdates { force: true });
    backend.send(Command::Detect);
    let action = match args.as_slice() {
        [verb, app] if verb == "install" => Some(Command::Install { app: app.clone() }),
        [verb, app] if verb == "uninstall" => Some(Command::Uninstall { app: app.clone() }),
        _ => None,
    };
    let (mut checked, mut sent) = (false, action.is_none());
    let mut action = action;
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(300) {
        std::thread::sleep(Duration::from_millis(300));
        for event in backend.poll() {
            match &event {
                Event::Latest { app, latest, from } => println!("latest   {app}: {latest:?} (from {})", from.label()),
                Event::Detected { app, installed } => println!("detected {app}: {installed:?}"),
                Event::Installable { app, installable } => println!("installable {app}: {installable}"),
                Event::Occupied { app, path } => println!("occupied {app}: {path}"),
                Event::JobFinished { app, result } => {
                    println!("JOB DONE {app}: {result:?}");
                    return;
                }
                Event::CheckFinished { .. } => checked = true,
                _ => {}
            }
        }
        if checked && !sent {
            sent = true;
            if let Some(cmd) = action.take() {
                backend.send(cmd);
            }
        }
        if checked && sent && matches!(args.len(), 0) {
            for app in ["flowgram", "voxflow", "mailflow"] {
                println!("download_size {app}: {:?}", backend.download_size(app));
            }
            return;
        }
    }
    println!("timed out");
}

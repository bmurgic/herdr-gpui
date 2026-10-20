//! Requires an active native desktop and an explicitly selected daemon executable.
//! The daemon and its sandbox are POSIX, so the harness compiles on Unix only.
#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::expect_used)]
#[path = "../../test-support/sandbox.rs"]
mod sandbox;

#[cfg(target_os = "macos")]
#[path = "support/clipboard_guard.rs"]
mod clipboard_guard;

use sandbox::{Sandbox, daemon_binary, stop_children};
use std::{
    ffi::OsString,
    fs,
    path::Path,
    process::{Child, Command, ExitStatus},
    thread,
    time::{Duration, Instant},
};

struct Isolated {
    sandbox: Sandbox,
    daemon: Option<Child>,
    gui: Option<Child>,
}

impl Isolated {
    /// Waits up to `timeout` for the GUI to exit, failing when the daemon dies
    /// first, and returns the GUI's exit status.
    fn wait_for_gui(&mut self, timeout: Duration) -> ExitStatus {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(status) = self.gui.as_mut().unwrap().try_wait().unwrap() {
                return status;
            }
            assert!(
                Instant::now() < deadline,
                "native GUI timeout (active desktop required)"
            );
            assert!(
                self.daemon.as_mut().unwrap().try_wait().unwrap().is_none(),
                "daemon died while GUI was running"
            );
            thread::sleep(Duration::from_millis(50));
        }
    }
}

fn gui_command(sandbox: &Sandbox, mut env: impl FnMut(&str) -> Option<OsString>) -> Command {
    let mut command = sandbox.command(env!("CARGO_BIN_EXE_herdr-gpui"), "gui.log");
    // Only the GUI needs desktop transport; keep HOME, XDG and daemon paths private.
    for name in ["DISPLAY", "WAYLAND_DISPLAY", "XAUTHORITY"] {
        if let Some(mut value) = env(name) {
            if name == "WAYLAND_DISPLAY"
                && !value.is_empty()
                && Path::new(&value).is_relative()
                && let Some(runtime) =
                    env("XDG_RUNTIME_DIR").filter(|dir| Path::new(dir).is_absolute())
            {
                // Relative Wayland sockets belong to the parent desktop, not our runtime dir.
                value = Path::new(&runtime).join(value).into_os_string();
            }
            command.env(name, value);
        }
    }
    command
}

#[test]
#[ignore = "requires active native desktop; GUI-only fixtures, no daemon"]
fn native_sidebar() {
    native_fixture(Fixture::Sidebar);
}

#[cfg(target_os = "linux")]
#[test]
#[ignore = "requires active Linux desktop; GUI-only menu fixtures, no daemon"]
fn native_application_menus() {
    native_fixture(Fixture::ApplicationMenus);
}

/// Separate from `native_sidebar`, whose earlier old-row checks would stop
/// it first. `just test-sidebar` runs both, as its filter matches this name.
#[cfg(target_os = "macos")]
#[test]
#[ignore = "requires active native desktop; GUI-only fixtures, no daemon"]
fn native_sidebar_session_rows() {
    native_fixture(Fixture::SessionRows);
}

/// Settings > General > Sessions, clicked natively. Its own fixture, outside
/// `native_sidebar`, whose earlier checks would stop it. It leaves out the
/// title model field's Enter save, which needs a key window macOS will not
/// give a background-launched app; see settings_window/native/sessions.rs.
#[cfg(target_os = "macos")]
#[test]
#[ignore = "requires active native desktop; GUI-only fixtures, no daemon"]
fn native_settings_sessions() {
    native_fixture(Fixture::SettingsSessions);
}

#[test]
#[ignore = "requires active native desktop; intentionally invalid paint fixture, no daemon"]
fn native_sidebar_probe_failure_exits_without_aborting() {
    native_fixture(Fixture::InvalidChildWidth);
}

#[test]
#[ignore = "requires active native desktop; notification fixtures, no daemon"]
fn native_notifications() {
    native_fixture(Fixture::Notifications);
}

#[cfg(target_os = "linux")]
#[test]
#[ignore = "requires active Wayland desktop; input shutdown fixtures, no daemon"]
fn native_input_shutdown_terminal() {
    native_input_shutdown("terminal");
}

#[cfg(target_os = "linux")]
#[test]
#[ignore = "requires active Wayland desktop; input shutdown fixtures, no daemon"]
fn native_input_shutdown_dialog() {
    native_input_shutdown("dialog");
}

#[cfg(target_os = "linux")]
#[test]
#[ignore = "requires active Wayland desktop; input shutdown fixtures, no daemon"]
fn native_input_shutdown_search() {
    native_input_shutdown("search");
}

#[cfg(target_os = "linux")]
fn native_input_shutdown(field: &'static str) {
    assert!(
        std::env::var_os("WAYLAND_DISPLAY").is_some(),
        "Wayland required"
    );
    native_fixture(Fixture::InputShutdown(field));
}

enum Fixture {
    Sidebar,
    #[cfg(target_os = "linux")]
    ApplicationMenus,
    Notifications,
    #[cfg(target_os = "macos")]
    SessionRows,
    #[cfg(target_os = "macos")]
    SettingsSessions,
    InvalidChildWidth,
    #[cfg(target_os = "linux")]
    InputShutdown(&'static str),
}

fn native_fixture(fixture: Fixture) {
    let mut isolated = Isolated {
        sandbox: Sandbox::new(),
        daemon: None,
        gui: None,
    };
    let mut command = gui_command(&isolated.sandbox, |name| std::env::var_os(name));
    if let Some(path) = std::env::var_os("HERDR_TEST_SETTINGS_CAPTURE") {
        command.env("HERDR_TEST_SETTINGS_CAPTURE", path);
    }
    if matches!(fixture, Fixture::Sidebar | Fixture::InvalidChildWidth) {
        keep_old_rows(&isolated.sandbox);
    }
    match fixture {
        Fixture::Sidebar => {}
        #[cfg(target_os = "linux")]
        Fixture::ApplicationMenus => {
            command.env("HERDR_TEST_APPLICATION_MENUS", "1");
        }
        Fixture::Notifications => {
            command.env("HERDR_TEST_NOTIFICATIONS_ONLY", "1");
        }
        #[cfg(target_os = "macos")]
        Fixture::SessionRows => {
            command.env("HERDR_TEST_SESSION_ROWS", "1");
        }
        #[cfg(target_os = "macos")]
        Fixture::SettingsSessions => {
            seed_sessions(&isolated.sandbox);
            command.env("HERDR_TEST_SETTINGS_SESSIONS", "1");
        }
        Fixture::InvalidChildWidth => {
            command.env("HERDR_TEST_SIDEBAR_PROBE_FAILURE", "1");
        }
        #[cfg(target_os = "linux")]
        Fixture::InputShutdown(field) => {
            command.env_remove("DISPLAY");
            command.env("HERDR_TEST_INPUT_SHUTDOWN", field);
            command.env("LEAK_BACKTRACE", "1");
        }
    }
    isolated.gui = Some(command.arg("--sidebar-test").spawn().unwrap());
    let gui = isolated.gui.as_mut().unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    while gui.try_wait().unwrap().is_none() {
        if Instant::now() >= deadline {
            let _ = gui.kill();
            let _ = gui.wait();
            panic!("native sidebar timeout");
        }
        thread::sleep(Duration::from_millis(50));
    }
    let status = gui.wait().unwrap();
    let log = fs::read_to_string(isolated.sandbox.dir.join("gui.log")).unwrap();
    eprintln!("{log}");
    if matches!(fixture, Fixture::InvalidChildWidth) {
        assert_eq!(
            status.code(),
            Some(1),
            "probe must fail without a signal: {status}"
        );
        assert!(
            log.contains("native paint probe \"sidebar-child\""),
            "{log}"
        );
        assert!(log.contains("child label width: actual"), "{log}");
        assert!(log.contains("expected"), "{log}");
        assert!(!log.contains("panicked at"), "{log}");
        assert!(!log.contains("SIDEBAR native PASS:"), "{log}");
        return;
    }
    assert!(status.success(), "native sidebar failed: {status}");
    assert!(log.contains(match fixture {
        #[cfg(target_os = "linux")]
        Fixture::ApplicationMenus => "APPLICATION MENUS native PASS",
        Fixture::Notifications => "NOTIFICATIONS native PASS:",
        #[cfg(target_os = "macos")]
        Fixture::SessionRows => "SESSION ROWS native PASS:",
        #[cfg(target_os = "macos")]
        Fixture::SettingsSessions => "SETTINGS SESSIONS native PASS:",
        #[cfg(target_os = "linux")]
        Fixture::InputShutdown(_) => "INPUT shutdown ready:",
        _ => "SIDEBAR native PASS:",
    }));
    #[cfg(target_os = "macos")]
    if matches!(fixture, Fixture::SettingsSessions) {
        assert_sessions_saved(&isolated.sandbox);
    }
}

/// The opposite of every value the Sessions fixture's clicks save, so each
/// click has to change the file.
#[cfg(target_os = "macos")]
fn seed_sessions(sandbox: &Sandbox) {
    let path = sandbox.dir.join("config.toml");
    let mut config = fs::read_to_string(&path).unwrap();
    config.push_str(
        "[ui]\nsession_sidebar = true\n[session_titles]\nenabled = false\nagent = \"claude\"\n",
    );
    fs::write(path, config).unwrap();
}

/// What the Sessions fixture saved reached the sandbox's own config file.
#[cfg(target_os = "macos")]
fn assert_sessions_saved(sandbox: &Sandbox) {
    let config = fs::read_to_string(sandbox.dir.join("config.toml")).unwrap();
    let config: toml::Table = toml::from_str(&config).unwrap();
    let ui = config["ui"].as_table().unwrap();
    let titles = config["session_titles"].as_table().unwrap();
    assert_eq!(ui["session_sidebar"].as_bool(), Some(false), "{config:?}");
    assert_eq!(titles["enabled"].as_bool(), Some(true), "{config:?}");
    assert_eq!(titles["agent"].as_str(), Some("codex"), "{config:?}");
    assert_eq!(
        titles
            .get("model")
            .and_then(toml::Value::as_str)
            .unwrap_or(""),
        "",
        "{config:?}"
    );
}

/// These fixtures' glyph checks measure the old agent row. Their Settings
/// phase reloads this file, which replaces the window's own override, so the
/// file must turn session rows off too.
fn keep_old_rows(sandbox: &Sandbox) {
    let path = sandbox.dir.join("config.toml");
    let mut config = fs::read_to_string(&path).unwrap();
    config.push_str("[ui]\nsession_sidebar = false\n");
    fs::write(path, config).unwrap();
}

impl Drop for Isolated {
    fn drop(&mut self) {
        stop_children([&mut self.gui, &mut self.daemon]);
        for name in ["gui.log", "daemon.log"] {
            if let Ok(log) = fs::read_to_string(self.sandbox.dir.join(name)) {
                eprintln!("{name} (last 40 lines, at most 600 chars each):");
                let lines = log.lines().rev().take(40).collect::<Vec<_>>();
                for line in lines.into_iter().rev() {
                    eprintln!("{}", line.chars().take(600).collect::<String>());
                }
            }
        }
    }
}

#[test]
#[ignore = "requires active desktop and explicit HERDR_TEST_BINARY; launches a native GUI and isolated daemon"]
fn native_gui_live() {
    let binary = daemon_binary();
    #[cfg(target_os = "macos")]
    let _clipboard = clipboard_guard::ClipboardGuard::acquire();
    let mut isolated = Isolated {
        sandbox: Sandbox::new(),
        daemon: None,
        gui: None,
    };
    let socket = isolated.sandbox.socket();
    let mut daemon = isolated.sandbox.command(binary, "daemon.log");
    // macOS /bin/sh otherwise edits UTF-8 input in the C locale, interpreting
    // high-bit bytes as editing keys before the selection fixture can run.
    #[cfg(target_os = "macos")]
    daemon.env("LC_ALL", "en_US.UTF-8");
    isolated.daemon = Some(daemon.arg("server").spawn().unwrap());
    isolated
        .sandbox
        .wait_for_daemon(isolated.daemon.as_mut().unwrap(), Duration::from_secs(20));
    let mut gui_command = gui_command(&isolated.sandbox, |name| std::env::var_os(name));
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::write(
            isolated.sandbox.dir.join("clipboard_capture.py"),
            include_str!("support/clipboard_capture.py"),
        )
        .unwrap();
        let tools = isolated.sandbox.dir.join("bin");
        fs::create_dir(&tools).unwrap();
        let ssh = tools.join("ssh");
        fs::write(&ssh, include_str!("support/clipboard_ssh.py")).unwrap();
        fs::set_permissions(&ssh, fs::Permissions::from_mode(0o700)).unwrap();
        gui_command.env(
            "PATH",
            format!("{}:/usr/bin:/bin:/usr/sbin:/sbin", tools.display()),
        );
    }
    isolated.gui = Some(
        gui_command
            .arg("--socket")
            .arg(&socket)
            .arg("--integration-test")
            .spawn()
            .unwrap(),
    );
    let status = isolated.wait_for_gui(Duration::from_secs(120));
    assert!(status.success(), "GUI failed: {status}");
    let log = fs::read_to_string(isolated.sandbox.dir.join("gui.log")).unwrap();
    assert!(
        log.contains("GUI integration PASS:"),
        "GUI exited without completing harness"
    );
    assert!(
        log.contains("GUI input pipeline verified:"),
        "GUI did not verify native action, key, and text delivery"
    );
    #[cfg(target_os = "macos")]
    assert!(
        log.contains("GUI clipboard native PASS: local")
            && log.contains("GUI clipboard native PASS: remote"),
        "native clipboard matrix did not complete"
    );
    #[cfg(target_os = "macos")]
    assert!(
        log.contains("GUI CJK selection verified:"),
        "GUI did not verify CJK native drag, clipboard, and paste byte readback"
    );
    assert!(
        log.contains("GUI second window verified:") && log.contains("windows=2 first_space="),
        "GUI did not verify a second window on its own space"
    );
    assert!(
        log.contains("GUI external workspace push verified:")
            && log.contains("bound_ms=3000 observation_poll_ms=100 unchanged_connection=true unchanged_focus=true no_refresh=true")
            && log.contains("3 workspaces / 4 tabs"),
        "GUI did not verify bounded external workspace delivery without refresh/reconnect"
    );
    assert!(
        isolated
            .daemon
            .as_mut()
            .unwrap()
            .try_wait()
            .unwrap()
            .is_none(),
        "GUI exit killed daemon"
    );
    eprintln!(
        "GUI exited successfully; isolated daemon is still alive; cleaning up only owned children"
    );
}

/// The text the seeded `new_session_command` types. A checklist closed with
/// `close` must leave it untyped.
#[cfg(target_os = "macos")]
const NEW_SESSION_MARKER: &str = "e2e-typed-marker";

/// Clicks the new session dialog's footer button, project row, `start`, and
/// `close` on a real window, against a repository whose `worktree.created`
/// hooks run for 20 seconds and fail. See `smoke_new_session.rs`.
#[cfg(target_os = "macos")]
#[test]
#[ignore = "requires active desktop and explicit HERDR_TEST_BINARY; launches a native GUI and isolated daemon"]
fn native_new_session_clicks_reach_their_rows() {
    native_new_session("project");
}

/// Clicks a folder suggestion on the dialog's folder page.
#[cfg(target_os = "macos")]
#[test]
#[ignore = "requires active desktop and explicit HERDR_TEST_BINARY; launches a native GUI and isolated daemon"]
fn native_new_session_folder_clicks_reach_their_rows() {
    native_new_session("folder");
}

#[cfg(target_os = "macos")]
fn native_new_session(walk: &str) {
    let binary = daemon_binary();
    let mut isolated = Isolated {
        sandbox: Sandbox::new(),
        daemon: None,
        gui: None,
    };
    let sandbox = &isolated.sandbox;
    let mut config = fs::read_to_string(sandbox.dir.join("config.toml")).unwrap();
    config.push_str(&format!(
        "[ui]\nnew_session_command = \"printf {NEW_SESSION_MARKER}\"\n"
    ));
    fs::write(sandbox.dir.join("config.toml"), config).unwrap();
    let repo = seed_repository(sandbox);
    link_setup_hook(sandbox, &binary, "e2e.hang", "sleep 20");
    link_setup_hook(sandbox, &binary, "e2e.fail", "exit 1");
    // Two folders under a home of their own, so the folder page lists exactly
    // these, in this order.
    let home = sandbox.dir.join("folders-home");
    fs::create_dir_all(home.join("photos")).unwrap();
    fs::create_dir_all(home.join("Projects")).unwrap();

    let socket = sandbox.socket();
    let mut daemon = sandbox.command(&binary, "daemon.log");
    daemon.env("LC_ALL", "en_US.UTF-8");
    isolated.daemon = Some(daemon.arg("server").spawn().unwrap());
    isolated
        .sandbox
        .wait_for_daemon(isolated.daemon.as_mut().unwrap(), Duration::from_secs(20));
    let mut command = gui_command(&isolated.sandbox, |name| std::env::var_os(name));
    command
        .env("HERDR_TEST_NEW_SESSION", walk)
        .env("HERDR_TEST_NEW_SESSION_REPO", &repo)
        .env("HERDR_TEST_NEW_SESSION_MARKER", NEW_SESSION_MARKER);
    if walk == "folder" {
        command.env("HOME", &home);
    }
    if let Some(path) = std::env::var_os("HERDR_TEST_NEW_SESSION_CAPTURE") {
        command.env("HERDR_TEST_NEW_SESSION_CAPTURE", path);
    }
    isolated.gui = Some(
        command
            .arg("--socket")
            .arg(&socket)
            .arg("--integration-test")
            .spawn()
            .unwrap(),
    );
    let status = isolated.wait_for_gui(Duration::from_secs(90));
    let log = fs::read_to_string(isolated.sandbox.dir.join("gui.log")).unwrap();
    eprintln!("{log}");
    assert!(status.success(), "native new session walk failed: {status}");
    let passed = format!("NEW SESSION native PASS: {}:", walk_name(walk));
    assert!(log.contains(&passed), "no {passed:?} line in the GUI log");
}

#[cfg(target_os = "macos")]
fn walk_name(walk: &str) -> &'static str {
    match walk {
        "project" => "Project",
        _ => "Folder",
    }
}

/// A git repository with one commit, so the daemon reports a repository root
/// for a workspace opened in it.
#[cfg(target_os = "macos")]
fn seed_repository(sandbox: &Sandbox) -> std::path::PathBuf {
    let repo = sandbox.dir.join("repo");
    fs::create_dir_all(&repo).unwrap();
    fs::write(repo.join("README"), "seed\n").unwrap();
    for args in [
        &["init", "-b", "main"][..],
        &["add", "README"],
        &["commit", "-m", "seed"],
    ] {
        let status = Command::new("git")
            .env_clear()
            .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
            .env("HOME", &sandbox.dir)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .args([
                "-c",
                "user.name=e2e",
                "-c",
                "user.email=e2e@example.invalid",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .current_dir(&repo)
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?} failed");
    }
    repo
}

/// Links a plugin into the sandbox's own registry whose `worktree.created`
/// hook runs `script`.
#[cfg(target_os = "macos")]
fn link_setup_hook(sandbox: &Sandbox, binary: &Path, id: &str, script: &str) {
    let root = sandbox.dir.join(format!("plugin-{id}"));
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("herdr-plugin.toml"),
        format!(
            r#"id = "{id}"
name = "{id}"
version = "0.1.0"
min_herdr_version = "0.9.0"
platforms = ["linux", "macos"]

[[events]]
on = "worktree.created"
command = ["/bin/sh", "-c", "{script}"]
"#
        ),
    )
    .unwrap();
    let status = sandbox
        .command(binary, format!("plugin-link-{id}.log"))
        .args(["plugin", "link"])
        .arg(&root)
        .status()
        .unwrap();
    assert!(status.success(), "plugin link {id} failed");
}

#[test]
fn gui_desktop_environment_preserves_sandbox_isolation() {
    use std::{collections::BTreeMap, ffi::OsStr, os::unix::ffi::OsStrExt};

    let sandbox = Sandbox::new();
    let daemon = sandbox.command("/usr/bin/env", "daemon.log");
    let base = daemon.get_envs().collect::<BTreeMap<_, _>>();
    assert_eq!(
        base[OsStr::new("XDG_RUNTIME_DIR")],
        Some(sandbox.dir.as_os_str())
    );
    for name in ["DISPLAY", "WAYLAND_DISPLAY", "XAUTHORITY"] {
        assert!(!base.contains_key(OsStr::new(name)));
    }
    assert_eq!(
        gui_command(&sandbox, |_| None)
            .get_envs()
            .collect::<BTreeMap<_, _>>(),
        base
    );

    for (display, runtime, expected) in [
        (
            Some("wayland-1"),
            Some("/run/user/1000"),
            Some("/run/user/1000/wayland-1"),
        ),
        (
            Some("nested/wayland-1"),
            Some("/run/user/1000"),
            Some("/run/user/1000/nested/wayland-1"),
        ),
        (
            Some("/desktop/wayland-1"),
            Some("/run/user/1000"),
            Some("/desktop/wayland-1"),
        ),
        (Some("/desktop/wayland-1"), None, Some("/desktop/wayland-1")),
        (Some("wayland-1"), None, Some("wayland-1")),
        (Some("wayland-1"), Some("relative"), Some("wayland-1")),
        (Some("wayland-1"), Some(""), Some("wayland-1")),
        (Some(""), Some("/run/user/1000"), Some("")),
        (None, Some("/run/user/1000"), None),
    ] {
        let command = gui_command(&sandbox, |name| match name {
            "DISPLAY" => Some(":42".into()),
            "XAUTHORITY" => Some("/desktop/auth".into()),
            "WAYLAND_DISPLAY" => display.map(Into::into),
            "XDG_RUNTIME_DIR" => runtime.map(Into::into),
            _ => panic!("must not inherit {name}"),
        });
        let mut expected_env = base.clone();
        expected_env.insert(OsStr::new("DISPLAY"), Some(OsStr::new(":42")));
        expected_env.insert(OsStr::new("XAUTHORITY"), Some(OsStr::new("/desktop/auth")));
        if let Some(expected) = expected {
            expected_env.insert(OsStr::new("WAYLAND_DISPLAY"), Some(OsStr::new(expected)));
        }
        assert_eq!(command.get_envs().collect::<BTreeMap<_, _>>(), expected_env);
        assert_eq!(command.get_current_dir(), daemon.get_current_dir());
    }

    let command = gui_command(&sandbox, |name| match name {
        "WAYLAND_DISPLAY" => Some(OsStr::from_bytes(b"wayland-\xff").to_owned()),
        "XDG_RUNTIME_DIR" => Some(OsStr::from_bytes(b"/run/user/\xfe").to_owned()),
        _ => None,
    });
    let env = command.get_envs().collect::<BTreeMap<_, _>>();
    assert_eq!(
        env[OsStr::new("WAYLAND_DISPLAY")],
        Some(OsStr::from_bytes(b"/run/user/\xfe/wayland-\xff"))
    );
}

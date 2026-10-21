//! The native new session walks: a real window clicked with AppKit mouse
//! events and keys sent through the window's own dispatch, against a sandbox
//! daemon this harness starts. `smoke_new_session.rs` runs each walk; this
//! file seeds its folders, hooks, and config, and does to the daemon what a
//! walk asks for, on the exact process it started.
use super::*;
use std::{ffi::OsStr, path::PathBuf};

/// The text the seeded `new_session_command` types in the `project` and
/// `folder` walks. A checklist closed with `close` must leave it untyped.
const NEW_SESSION_MARKER: &str = "e2e-typed-marker";
/// qa.md's seeded command and the line it prints, for the qa walks.
const QA_COMMAND: &str = "printf qa-after-setup";
const QA_MARKER: &str = "qa-after-setup";

/// The hook that runs for 30 seconds. It first closes every descriptor past
/// stdio. The daemon starts each hook from its own thread. On macOS a pipe end
/// one thread has just created can leak into the process another thread
/// starts at the same moment, and a leaked end keeps the first hook's output
/// open, so its row would stay running until this hook exits. A scratch
/// program with the daemon's spawn pattern showed that leak (the quick hook
/// took as long as the sleeping one in 3 of 1000 pairs under CPU load, 0 of
/// 1000 with this prefix). The one native stall this guards against was not
/// traced to it.
const HANG_HOOK: &str =
    r"for f in /dev/fd/*; do n=${f##*/}; [ $n -gt 2 ] && eval exec $n\>\&-; done; sleep 30";
/// qa.md's two setup plugins.
const LONG_SETUP: (&str, &str) = ("qa.long-setup", HANG_HOOK);
const FAIL_SETUP: (&str, &str) = (
    "qa.fail-setup",
    "echo setup-out; echo setup-err >&2; exit 1",
);

/// Clicks the new session dialog's footer button, project row, `start`,
/// `close`, and then `start anyway` on a second session, on a real window,
/// against a repository whose `worktree.created` hooks run for 30 seconds and
/// fail.
#[test]
#[ignore = "requires active desktop and explicit HERDR_TEST_BINARY; launches a native GUI and isolated daemon"]
fn native_new_session_clicks_reach_their_rows() {
    native_new_session("project");
}

/// Clicks a folder suggestion on the dialog's folder page.
#[test]
#[ignore = "requires active desktop and explicit HERDR_TEST_BINARY; launches a native GUI and isolated daemon"]
fn native_new_session_folder_clicks_reach_their_rows() {
    native_new_session("folder");
}

/// qa.md's "Open the new-session dialog from the Agents tab", steps 1 to 7,
/// and the palette's `New Session`.
#[test]
#[ignore = "requires active desktop and explicit HERDR_TEST_BINARY; launches a native GUI and isolated daemon"]
fn native_new_session_agents_tab_keys() {
    native_new_session("agents");
}

/// Step 1 of qa.md's "Change the New session command in Settings": the row's
/// place and text, and its screenshots.
#[test]
#[ignore = "requires active desktop and explicit HERDR_TEST_BINARY; launches a native GUI and isolated daemon"]
fn native_new_session_settings_command() {
    native_new_session("settings");
}

/// qa.md's "Review the confirm page for a repository", steps 1 to 7.
#[test]
#[ignore = "requires active desktop and explicit HERDR_TEST_BINARY; launches a native GUI and isolated daemon"]
fn native_new_session_confirm_keys() {
    native_new_session("confirm");
}

/// qa.md's "Check another folder", steps 1 to 3 and 5 to 7.
#[test]
#[ignore = "requires active desktop and explicit HERDR_TEST_BINARY; launches a native GUI and isolated daemon"]
fn native_new_session_folder_keys() {
    native_new_session("folder-keys");
}

/// qa.md's "Start a session in a new worktree", steps 1 to 4.
#[test]
#[ignore = "requires active desktop and explicit HERDR_TEST_BINARY; launches a native GUI and isolated daemon"]
fn native_new_session_start_types_in_new_tab() {
    native_new_session("start");
}

/// qa.md's "Recover from a failed start", steps 1 to 3.
#[test]
#[ignore = "requires active desktop and explicit HERDR_TEST_BINARY; launches a native GUI and isolated daemon"]
fn native_new_session_failed_start_retries() {
    native_new_session("failed-start");
}

/// qa.md's "Lose the connection while the dialog is open", steps 1 to 12. The
/// harness pauses, resumes, stops, and restarts the exact daemon it started.
#[test]
#[ignore = "requires active desktop and explicit HERDR_TEST_BINARY; launches a native GUI and isolated daemon"]
fn native_new_session_outage_keeps_dialog() {
    native_new_session("outage");
}

/// qa.md's "Wait for a setup hook": the timer read, then steps 2 and 3.
#[test]
#[ignore = "requires active desktop and explicit HERDR_TEST_BINARY; launches a native GUI and isolated daemon"]
fn native_new_session_setup_wait() {
    native_new_session("setup-wait");
}

/// Step 2 of qa.md's "See a failed setup hook": the failed row's log.
#[test]
#[ignore = "requires active desktop and explicit HERDR_TEST_BINARY; launches a native GUI and isolated daemon"]
fn native_new_session_setup_failed_log() {
    native_new_session("setup-failed");
}

/// What a walk starts from, beyond the sandbox's own config.
struct Seed {
    /// The `[ui] new_session_command` the config sets.
    command: &'static str,
    /// The text that command prints.
    marker: &'static str,
    /// The `worktree.created` plugins linked before the daemon starts.
    hooks: &'static [(&'static str, &'static str)],
    /// Whether the walk's folders are qa.md's `repos/demo`, `notes`, and
    /// `repos/demo-fix`, rather than the older walks' `repo`.
    is_qa: bool,
    /// How long the walk may run.
    timeout: Duration,
}

fn seed(walk: &str) -> Seed {
    let older = Seed {
        command: "printf e2e-typed-marker",
        marker: NEW_SESSION_MARKER,
        hooks: &[("e2e.hang", HANG_HOOK), ("e2e.fail", "exit 1")],
        is_qa: false,
        timeout: Duration::from_secs(90),
    };
    let qa = Seed {
        command: QA_COMMAND,
        marker: QA_MARKER,
        hooks: &[],
        is_qa: true,
        timeout: Duration::from_secs(90),
    };
    match walk {
        "project" | "folder" => older,
        "setup-wait" => Seed {
            hooks: &[LONG_SETUP],
            ..qa
        },
        "setup-failed" => Seed {
            hooks: &[LONG_SETUP, FAIL_SETUP],
            ..qa
        },
        // Two waits for herdr-client's 60 second request timeout, and two
        // daemon restarts.
        "outage" => Seed {
            hooks: &[LONG_SETUP],
            timeout: Duration::from_secs(360),
            ..qa
        },
        _ => qa,
    }
}

fn native_new_session(walk: &str) {
    let binary = daemon_binary();
    let seed = seed(walk);
    let mut isolated = Isolated {
        sandbox: Sandbox::new(),
        daemon: None,
        gui: None,
    };
    let sandbox = &isolated.sandbox;
    let mut config = fs::read_to_string(sandbox.dir.join("config.toml")).unwrap();
    if seed.is_qa {
        config.push_str("[session_titles]\nenabled = false\n");
    }
    config.push_str(&format!(
        "[ui]\nnew_session_command = \"{}\"\n",
        seed.command
    ));
    fs::write(sandbox.dir.join("config.toml"), config).unwrap();
    // The daemon reports folders by their resolved path (`/private/var/...`
    // on macOS), so the qa walks seed under it and give the window it as
    // home, where qa.md's `~/repos/demo` comes from.
    let qa_home = fs::canonicalize(&sandbox.dir).unwrap();
    let folders = if seed.is_qa {
        qa_folders(sandbox, &qa_home)
    } else {
        Folders {
            demo: seed_repository(sandbox, &sandbox.dir.join("repo")),
            notes: None,
            linked: None,
        }
    };
    for (id, script) in seed.hooks {
        link_setup_hook(sandbox, &binary, id, script);
    }
    // Two folders under a home of their own, so the folder page lists exactly
    // these, in this order.
    let home = sandbox.dir.join("folders-home");
    fs::create_dir_all(home.join("photos")).unwrap();
    fs::create_dir_all(home.join("Projects")).unwrap();
    let control = sandbox.dir.join("daemon-control");
    fs::create_dir_all(&control).unwrap();

    // The daemon binds the standard session socket for the sandbox's
    // `XDG_CONFIG_HOME`, `config/herdr/herdr-client.sock`, whichever build
    // (release or development) it is. The GUI dials exactly that path, so the
    // connector's own check finds the daemon on the user's standard endpoint
    // and trusts it as local, which the folder page needs. A socket anywhere
    // else would never be trusted. The integration mode requires `--socket`.
    let socket = sandbox.dir.join("config/herdr/herdr-client.sock");
    assert!(
        socket.as_os_str().len() < 104,
        "standard socket path too long for Unix socket; set HERDR_TEST_TMPDIR to a shorter parent"
    );
    isolated.daemon = Some(spawn_daemon(sandbox, &binary, "daemon.log"));
    isolated.sandbox.wait_for_daemon_at(
        &socket,
        isolated.daemon.as_mut().unwrap(),
        Duration::from_secs(20),
    );
    let mut command = gui_command(&isolated.sandbox, |name| std::env::var_os(name));
    command
        .env("HERDR_TEST_NEW_SESSION", walk)
        .env("HERDR_TEST_NEW_SESSION_REPO", &folders.demo)
        .env("HERDR_TEST_NEW_SESSION_MARKER", seed.marker)
        .env("HERDR_TEST_NEW_SESSION_SANDBOX", &isolated.sandbox.dir)
        .env("HERDR_TEST_NEW_SESSION_CONTROL", &control);
    if let Some(notes) = &folders.notes {
        command.env("HERDR_TEST_NEW_SESSION_NOTES", notes);
    }
    if let Some(linked) = &folders.linked {
        command.env("HERDR_TEST_NEW_SESSION_LINKED", linked);
    }
    if walk == "folder" {
        command.env("HOME", &home);
    } else if seed.is_qa {
        // The same config file, named under the resolved home.
        command
            .env("HOME", &qa_home)
            .env("HERDR_CONFIG_PATH", qa_home.join("config.toml"));
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
    let status = serve_walk(&mut isolated, &binary, &socket, &control, seed.timeout);
    let log = fs::read_to_string(isolated.sandbox.dir.join("gui.log")).unwrap();
    eprintln!("{log}");
    assert!(
        status.success(),
        "native new session walk {walk} failed: {status}"
    );
    let passed = format!("NEW SESSION native PASS: {walk}:");
    assert!(log.contains(&passed), "no {passed:?} line in the GUI log");
}

/// Starts the daemon as every walk does, logging to `log`.
fn spawn_daemon(sandbox: &Sandbox, binary: &Path, log: &str) -> Child {
    let mut daemon = sandbox.command(binary, log);
    daemon
        .env(
            "HERDR_SOCKET_PATH",
            sandbox.dir.join("config/herdr/herdr.sock"),
        )
        .env_remove("HERDR_CLIENT_SOCKET_PATH")
        .env("LC_ALL", "en_US.UTF-8");
    daemon.arg("server").spawn().unwrap()
}

/// Waits up to `timeout` for the GUI to exit, and meanwhile does each daemon
/// request the walk writes into `control`. The daemon must stay alive unless
/// the walk stopped it.
fn serve_walk(
    isolated: &mut Isolated,
    binary: &Path,
    socket: &Path,
    control: &Path,
    timeout: Duration,
) -> ExitStatus {
    let deadline = Instant::now() + timeout;
    let mut next = 1;
    let mut is_stopped = false;
    let mut restarts = 0;
    loop {
        if let Some(status) = isolated.gui.as_mut().unwrap().try_wait().unwrap() {
            if let Some(daemon) = isolated.daemon.as_ref().filter(|_| !is_stopped) {
                // A walk that fails while the daemon is paused must not leave it so.
                signal(daemon.id(), "CONT");
            }
            return status;
        }
        assert!(
            Instant::now() < deadline,
            "native GUI timeout (active desktop required)"
        );
        if !is_stopped {
            assert!(
                isolated
                    .daemon
                    .as_mut()
                    .unwrap()
                    .try_wait()
                    .unwrap()
                    .is_none(),
                "daemon died while GUI was running"
            );
        }
        let request = control.join(format!("request-{next}"));
        if let Ok(action) = fs::read_to_string(&request) {
            let pid = isolated.daemon.as_ref().unwrap().id();
            let answer = match action.as_str() {
                "pause" => signal(pid, "STOP"),
                "resume" => signal(pid, "CONT"),
                "stop" => {
                    let answer = signal(pid, "TERM");
                    is_stopped = true;
                    wait_for_exit(isolated.daemon.as_mut().unwrap(), answer)
                }
                "restart" => {
                    restarts += 1;
                    let log = format!("daemon-restart-{restarts}.log");
                    let mut daemon = spawn_daemon(&isolated.sandbox, binary, &log);
                    isolated.sandbox.wait_for_daemon_at(
                        socket,
                        &mut daemon,
                        Duration::from_secs(20),
                    );
                    let answer = format!("pid {}", daemon.id());
                    isolated.daemon = Some(daemon);
                    is_stopped = false;
                    answer
                }
                other => format!("error: unknown request {other:?}"),
            };
            eprintln!("daemon request {next} {action}: {answer}");
            let staged = control.join(format!("answer-{next}"));
            fs::write(&staged, &answer).unwrap();
            fs::rename(&staged, control.join(format!("done-{next}"))).unwrap();
            next += 1;
        }
        thread::sleep(Duration::from_millis(50));
    }
}

/// Sends `SIG<name>` to the exact process `pid`, which this harness started.
fn signal(pid: u32, name: &str) -> String {
    let status = Command::new("/bin/kill")
        .args(["-s", name, &pid.to_string()])
        .status()
        .unwrap();
    if status.success() {
        format!("pid {pid}")
    } else {
        format!("error: kill -s {name} {pid} exited {status}")
    }
}

fn wait_for_exit(daemon: &mut Child, answer: String) -> String {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if let Some(status) = daemon.try_wait().unwrap() {
            return format!("{answer} exited {status}");
        }
        if Instant::now() >= deadline {
            return format!("error: {answer} did not exit within 20 seconds");
        }
        thread::sleep(Duration::from_millis(20));
    }
}

/// The folders a walk's workspaces open in.
struct Folders {
    demo: PathBuf,
    notes: Option<PathBuf>,
    linked: Option<PathBuf>,
}

/// qa.md's folders: the repository `repos/demo` with a subfolder `src`, the
/// linked worktree `repos/demo-fix` of it, and `notes`, which is in no
/// repository.
fn qa_folders(sandbox: &Sandbox, root: &Path) -> Folders {
    let demo = seed_repository(sandbox, &root.join("repos/demo"));
    fs::create_dir_all(demo.join("src")).unwrap();
    let linked = root.join("repos/demo-fix");
    git(
        sandbox,
        &demo,
        &[
            OsStr::new("worktree"),
            OsStr::new("add"),
            OsStr::new("-b"),
            OsStr::new("demo-fix"),
            linked.as_os_str(),
        ],
    );
    let notes = root.join("notes");
    fs::create_dir_all(&notes).unwrap();
    Folders {
        demo,
        notes: Some(notes),
        linked: Some(linked),
    }
}

/// A git repository with one commit on `main` at `repo`, so the daemon
/// reports a repository root for a workspace opened in it.
fn seed_repository(sandbox: &Sandbox, repo: &Path) -> PathBuf {
    fs::create_dir_all(repo).unwrap();
    fs::write(repo.join("README"), "seed\n").unwrap();
    for args in [
        &["init", "-b", "main"][..],
        &["add", "README"],
        &["commit", "-m", "seed"],
    ] {
        git(sandbox, repo, args);
    }
    repo.to_path_buf()
}

/// Runs git in `repo` with a cleared environment and the sandbox as home.
fn git(sandbox: &Sandbox, repo: &Path, args: &[impl AsRef<OsStr>]) {
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
        .current_dir(repo)
        .status()
        .unwrap();
    let args: Vec<_> = args.iter().map(AsRef::as_ref).collect();
    assert!(status.success(), "git {args:?} failed");
}

/// Links a plugin into the sandbox's own registry whose `worktree.created`
/// hook runs `script`.
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
command = ["/bin/sh", "-c", '''{script}''']
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

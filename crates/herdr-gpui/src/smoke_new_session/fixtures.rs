//! The qa walks' starting state, and the requests that pause, resume, stop,
//! and restart the sandbox daemon.
//!
//! The test harness seeds the folders: `HERDR_TEST_NEW_SESSION_REPO` is the
//! `demo` repository, `HERDR_TEST_NEW_SESSION_NOTES` is the `notes` folder, and
//! `HERDR_TEST_NEW_SESSION_LINKED` is `demo-fix`, a linked worktree of `demo`.
//! The walk opens a workspace in each from a second client, in qa.md's order.
//!
//! Only the harness owns the daemon process, so a walk asks it to signal the
//! daemon by writing `request-<n>` into `HERDR_TEST_NEW_SESSION_CONTROL`, and
//! the harness answers in `done-<n>` once the daemon has paused, resumed,
//! exited, or listens again.
use super::*;

const NOTES_VARIABLE: &str = "HERDR_TEST_NEW_SESSION_NOTES";
const LINKED_VARIABLE: &str = "HERDR_TEST_NEW_SESSION_LINKED";
const CONTROL_VARIABLE: &str = "HERDR_TEST_NEW_SESSION_CONTROL";
/// The labels qa.md's shared starting state gives the workspaces.
pub(super) const DEMO: &str = "demo";
pub(super) const NOTES: &str = "notes";
pub(super) const LINKED: &str = "demo-fix";
/// The project page's last entry.
pub(super) const OTHER_FOLDER: &str = "Other folder…";
/// The command qa.md's config seeds as `ui.new_session_command`.
pub(super) const QA_COMMAND: &str = "printf qa-after-setup";
/// The line `QA_COMMAND` prints.
pub(super) const QA_OUTPUT: &str = "qa-after-setup";
/// How long the harness may take to act on a daemon request. A restart waits
/// for the new daemon's socket, which the harness allows 20 seconds.
const CONTROL_DEADLINE: Duration = Duration::from_secs(30);

/// The workspaces qa.md starts from.
pub(super) struct Fixtures {
    pub(super) demo: PathBuf,
    pub(super) notes: PathBuf,
    pub(super) demo_id: String,
    /// Every workspace before the walk acts, so it can tell which it added.
    pub(super) known: Vec<String>,
    /// The second clients that made the workspaces. Each stays connected
    /// until the walk ends.
    _clients: Vec<ExternalWorkspace>,
}

/// What the harness does to the daemon it started.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DaemonAction {
    /// `kill -STOP` on its process id.
    Pause,
    /// `kill -CONT` on its process id.
    Resume,
    /// `kill` on its process id, then a wait until it has exited.
    Stop,
    /// The same command again, with the same sockets and state folder.
    Restart,
}

impl DaemonAction {
    fn name(self) -> &'static str {
        match self {
            Self::Pause => "pause",
            Self::Resume => "resume",
            Self::Stop => "stop",
            Self::Restart => "restart",
        }
    }
}

impl Driver {
    /// Opens `demo`, `notes`, and, when `with_linked` is set, `demo-fix`, and
    /// waits until the snapshot lists them with `demo`'s repository known and
    /// `demo-fix` marked as a linked worktree.
    pub(super) async fn fixtures(&self, cx: &mut AsyncApp, with_linked: bool) -> Result<Fixtures> {
        let demo = env_path(REPO_VARIABLE)?;
        let notes = env_path(NOTES_VARIABLE)?;
        let mut seeds = vec![(demo.clone(), DEMO), (notes.clone(), NOTES)];
        if with_linked {
            seeds.push((env_path(LINKED_VARIABLE)?, LINKED));
        }
        let mut clients = Vec::new();
        for (cwd, label) in &seeds {
            clients.push(self.create_workspace(cx, cwd, label).await?);
        }
        let demo_id = clients[0].id.clone();
        self.until(cx, "the seeded workspaces in the snapshot", |view| {
            let Some(snapshot) = view.live.snapshot.as_ref() else {
                return Ok(None);
            };
            let is_seeded = clients.iter().zip(&seeds).all(|(client, (_, label))| {
                snapshot.workspaces.iter().any(|workspace| {
                    workspace.workspace_id == client.id
                        && workspace.label == *label
                        && match *label {
                            DEMO => workspace
                                .repository
                                .as_ref()
                                .is_some_and(|repository| !repository.is_linked_worktree),
                            LINKED => workspace
                                .repository
                                .as_ref()
                                .is_some_and(|repository| repository.is_linked_worktree),
                            _ => true,
                        }
                })
            });
            Ok(is_seeded.then_some(()))
        })
        .await?;
        let known = self.workspace_ids(cx)?;
        Ok(Fixtures {
            demo,
            notes,
            demo_id,
            known,
            _clients: clients,
        })
    }

    /// Opens a workspace labeled `label` in `cwd` from a second client, as
    /// another Herdr client would, without focusing it.
    pub(super) async fn create_workspace(
        &self,
        cx: &mut AsyncApp,
        cwd: &FilePath,
        label: &str,
    ) -> Result<ExternalWorkspace> {
        let Ready {
            boot,
            target,
            options,
        } = self.ready(cx).await?;
        let params = serde_json::json!({"cwd": cwd, "focus": false, "label": label});
        let (sender, receiver) = mpsc::channel();
        std::thread::Builder::new()
            .name("new-session-walk-workspace".into())
            .spawn(move || {
                let _ = sender.send(create_workspace_from_second_client(
                    target, options, boot, params,
                ));
            })
            .context("spawning the workspace worker")?;
        self.until(
            cx,
            &format!("the second client's workspace.create reply for {label}"),
            |_| match receiver.try_recv() {
                Ok(created) => created.map(Some),
                Err(TryRecvError::Empty) => Ok(None),
                Err(error) => Err(error.into()),
            },
        )
        .await
    }

    /// Asks the harness to do `action` to the daemon and waits for its
    /// answer, which names the daemon's process id.
    pub(super) async fn daemon(&self, cx: &mut AsyncApp, action: DaemonAction) -> Result<String> {
        let folder = env_path(CONTROL_VARIABLE)?;
        let number = self.requests.get() + 1;
        self.requests.set(number);
        let request = folder.join(format!("request-{number}"));
        let staged = folder.join(format!("staged-{number}"));
        let name = action.name();
        // Written aside and renamed, so the harness never reads half a request.
        cx.background_executor()
            .spawn(async move {
                std::fs::write(&staged, name)?;
                std::fs::rename(&staged, &request)
            })
            .await
            .with_context(|| format!("writing daemon request {number} ({name})"))?;
        let done = folder.join(format!("done-{number}"));
        let deadline = Instant::now() + CONTROL_DEADLINE;
        loop {
            let path = done.clone();
            let answer = cx
                .background_executor()
                .spawn(async move { std::fs::read_to_string(path).ok() })
                .await;
            if let Some(answer) = answer {
                ensure!(
                    !answer.starts_with("error"),
                    "the harness could not {name} the daemon: {answer}"
                );
                eprintln!("NEW SESSION daemon {name}: {answer}");
                return Ok(answer);
            }
            ensure!(
                Instant::now() < deadline,
                "the harness did not {name} the daemon within {CONTROL_DEADLINE:?}"
            );
            self.timer.timer(POLL).await;
        }
    }
}

fn env_path(name: &str) -> Result<PathBuf> {
    let path = std::env::var_os(name)
        .map(PathBuf::from)
        .with_context(|| format!("{name} unset"))?;
    ensure!(path.is_absolute(), "{name} must be absolute, not {path:?}");
    Ok(path)
}

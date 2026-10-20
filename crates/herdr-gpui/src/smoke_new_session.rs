//! Native walk of the new session dialog against a real daemon, driven by
//! AppKit mouse events on the real window. The headless tests reach the dialog
//! through the test platform; this walk proves each click lands on the row
//! painted under it. `live_gui.rs` starts it with `HERDR_TEST_NEW_SESSION` set
//! to the walk's name.
//!
//! The `project` walk opens the dialog from the sidebar footer, picks a project
//! that is a git repository, starts a session with two `worktree.created`
//! hooks (one runs until killed, one exits 1), reads the checklist, and closes
//! it with the `close` button. Closing must focus the new workspace and must
//! not type the configured command. The `folder` walk clicks a folder
//! suggestion and checks that it fills the field and sends nothing.
use super::*;
use crate::{
    menu::NewSessionStage,
    sidebar::{layout_tests::PaintedProbes, native_tests::Target},
};
use anyhow::ensure;
use std::{
    path::PathBuf,
    sync::mpsc::{self, TryRecvError},
};

/// Names the walk to run: `project` or `folder`.
const WALK_VARIABLE: &str = "HERDR_TEST_NEW_SESSION";
/// The git repository the project walk's workspace opens in.
const REPO_VARIABLE: &str = "HERDR_TEST_NEW_SESSION_REPO";
/// The text the configured `new_session_command` types, which a closed
/// checklist must not.
const MARKER_VARIABLE: &str = "HERDR_TEST_NEW_SESSION_MARKER";
/// The sandbox directory the daemon, its socket, and the GUI's environment live
/// in. The walk refuses to send anything unless the socket the window dials is
/// inside it.
const SANDBOX_VARIABLE: &str = "HERDR_TEST_NEW_SESSION_SANDBOX";
/// A directory that receives a PNG of the window at each stage. Used only
/// when built with the `mockup` feature, which can render a window to an image.
const CAPTURE_VARIABLE: &str = "HERDR_TEST_NEW_SESSION_CAPTURE";
/// How long a click or a daemon reply may take before the walk fails.
const DEADLINE: Duration = Duration::from_secs(15);
/// How long the walk keeps watching for a command that must not be typed.
const QUIET_PERIOD: Duration = Duration::from_millis(1500);
const POLL: Duration = Duration::from_millis(20);
/// The label of the workspace the project walk creates.
const PROJECT_LABEL: &str = "demo";
/// The first line the shell prints in the test sandbox.
const SHELL_PROMPT: &str = "LIVE>";
/// The folder page's row for the second suggestion, and the field text a click
/// on it must leave. The test gives the home folder two children, so this is
/// the second row.
const FOLDER_ROW: &str = "Projects";
const FOLDER_TEXT: &str = "~/Projects";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Walk {
    Project,
    Folder,
}

impl Walk {
    pub(super) fn from_env() -> Option<Self> {
        match std::env::var(WALK_VARIABLE).ok()?.as_str() {
            "project" => Some(Self::Project),
            "folder" => Some(Self::Folder),
            _ => None,
        }
    }
}

pub(super) fn start(handle: WindowHandle<HerdrWindow>, walk: Walk, cx: &mut App) {
    EXIT_CODE.store(1, Ordering::SeqCst);
    cx.set_global(PaintedProbes::default());
    let timer = cx.background_executor().clone();
    cx.spawn(async move |cx| {
        let outcome = match Driver::new(handle, timer, cx).await {
            Ok(driver) => match walk {
                Walk::Project => driver.project(cx).await,
                Walk::Folder => driver.folder(cx).await,
            },
            Err(error) => Err(error),
        };
        match outcome {
            Ok(summary) => {
                eprintln!("NEW SESSION native PASS: {walk:?}: {summary}");
                std::process::exit(0);
            }
            Err(error) => {
                eprintln!("NEW SESSION native FAIL: {walk:?}: {error:#}");
                std::process::exit(1);
            }
        }
    })
    .detach();
}

/// What the walk needs from a window that has reached its daemon.
struct Ready {
    boot: String,
    target: ConnectTarget,
    options: ConnectOptions,
}

/// The window, the clock that paces it, and the AppKit handle that sends it
/// clicks.
struct Driver {
    handle: WindowHandle<HerdrWindow>,
    timer: BackgroundExecutor,
    target: Target,
}

impl Driver {
    async fn new(
        handle: WindowHandle<HerdrWindow>,
        timer: BackgroundExecutor,
        cx: &mut AsyncApp,
    ) -> Result<Self> {
        let target = handle.update(cx, |_, window, _| Target::acquire(window))??;
        let driver = Self {
            handle,
            timer,
            target,
        };
        driver.check_socket_is_in_sandbox(cx)?;
        Ok(driver)
    }

    /// Fails unless the window dials the sandbox daemon's standard session
    /// socket, inside the sandbox. The window then trusts the daemon as local
    /// by the connector's own check, which compares the dialed path with the
    /// standard path for the process's `XDG_CONFIG_HOME`. Both paths come from
    /// the window's environment; a lookup that ignored `$HOME` and the XDG
    /// variables could otherwise reach a user's real daemon. This runs before
    /// the walk sends anything.
    fn check_socket_is_in_sandbox(&self, cx: &mut AsyncApp) -> Result<()> {
        let root = std::env::var_os(SANDBOX_VARIABLE)
            .map(PathBuf::from)
            .with_context(|| format!("{SANDBOX_VARIABLE} unset"))?;
        ensure!(
            root.is_absolute() && root.is_dir(),
            "{SANDBOX_VARIABLE} must name an absolute directory, not {root:?}"
        );
        let target = self.handle.read_with(cx, |view, _| {
            view.endpoints[view.selected_endpoint]
                .connection
                .target
                .clone()
        })?;
        let ConnectTarget::Socket(dialed) = &target else {
            bail!("the window connects to {target:?}, not to a socket in the sandbox");
        };
        let standard = target
            .local_session_socket_path()
            .context("resolving the standard session socket")?;
        for (what, path) in [("dials", dialed), ("trusts", &standard)] {
            ensure!(
                path.starts_with(&root),
                "the window {what} {path:?}, which is outside the sandbox {root:?}"
            );
        }
        ensure!(
            *dialed == standard,
            "the window dials {dialed:?}, not the standard session socket {standard:?}"
        );
        Ok(())
    }

    /// The project walk, described at the top of this file.
    async fn project(&self, cx: &mut AsyncApp) -> Result<String> {
        let repo =
            std::env::var(REPO_VARIABLE).with_context(|| format!("{REPO_VARIABLE} unset"))?;
        let marker =
            std::env::var(MARKER_VARIABLE).with_context(|| format!("{MARKER_VARIABLE} unset"))?;
        let ready = self.ready(cx).await?;
        let known = self.workspace_ids(cx)?;

        let params = serde_json::json!({"cwd": repo, "focus": false, "label": PROJECT_LABEL});
        let (sender, receiver) = mpsc::channel();
        let Ready {
            boot,
            target,
            options,
        } = ready;
        std::thread::Builder::new()
            .name("new-session-walk-workspace".into())
            .spawn(move || {
                let _ = sender.send(create_workspace_from_second_client(
                    target, options, boot, params,
                ));
            })
            .context("spawning the workspace worker")?;
        // The second client stays connected until the walk ends.
        let project = self
            .until(
                cx,
                "the second client's workspace.create reply",
                |_| match receiver.try_recv() {
                    Ok(created) => created.map(Some),
                    Err(TryRecvError::Empty) => Ok(None),
                    Err(error) => Err(error.into()),
                },
            )
            .await?;
        self.until(cx, "the demo project in the snapshot", |view| {
            Ok(view.live.snapshot.as_ref().and_then(|snapshot| {
                snapshot
                    .workspaces
                    .iter()
                    .find(|workspace| workspace.workspace_id == project.id)
                    .filter(|workspace| {
                        workspace.label == PROJECT_LABEL && workspace.repository.is_some()
                    })
                    .map(|_| ())
            }))
        })
        .await?;

        self.capture_old_footer_page(cx).await?;
        self.open_dialog(cx).await?;
        self.click_text(cx, "the demo project row", |text| text == PROJECT_LABEL)
            .await?;
        self.stage(cx, NewSessionStage::Confirm).await?;
        self.capture(cx, "confirm")?;
        self.click_text(cx, "the start button", |text| text == "start")
            .await?;
        self.stage(cx, NewSessionStage::Checklist).await?;
        let running = self
            .painted_text(cx, "a running setup row", |text| {
                text.starts_with("Run setup: ")
            })
            .await?;
        let failed = self
            .painted_text(cx, "the failed setup row", |text| {
                text.starts_with("✗ Setup failed (exit 1)")
            })
            .await?;
        self.capture(cx, "checklist")?;

        self.click_text(cx, "the close button", |text| text == "close")
            .await?;
        self.until(cx, "the closed dialog", |view| {
            Ok((view.new_session_stage().is_none() && view.menu.page.is_none()).then_some(()))
        })
        .await?;
        let started = self
            .until(cx, "the new workspace's shell", |view| {
                Ok(shell_rows(view)
                    .filter(|(workspace, rows)| {
                        !known.contains(workspace)
                            && *workspace != project.id
                            && rows.iter().any(|row| row.contains(SHELL_PROMPT))
                    })
                    .map(|(workspace, _)| workspace))
            })
            .await?;
        self.capture(cx, "closed")?;
        // A command typed after the close would echo on the prompt line.
        let deadline = Instant::now() + QUIET_PERIOD;
        while Instant::now() < deadline {
            let typed = self.handle.read_with(cx, |view, _| {
                shell_rows(view)
                    .is_some_and(|(_, rows)| rows.iter().any(|row| row.contains(&marker)))
            })?;
            ensure!(
                !typed,
                "closing the checklist typed {marker:?} in {started}"
            );
            self.timer.timer(POLL).await;
        }
        Ok(format!(
            "clicks on the footer new button, the {PROJECT_LABEL:?} project row, start, and \
             close reached their rows; the checklist showed {running:?} and {failed:?}; close \
             focused {started} and typed nothing"
        ))
    }

    /// The folder walk, described at the top of this file.
    async fn folder(&self, cx: &mut AsyncApp) -> Result<String> {
        self.ready(cx).await?;
        // The folder page lists this computer's folders only for a daemon the
        // window trusts as local, which the connector decides from the socket.
        self.until(cx, "the window to trust the daemon as local", |view| {
            Ok(view.live.local_daemon_peer.then_some(()))
        })
        .await?;
        self.open_dialog(cx).await?;
        self.click_text(cx, "the Other folder row", |text| text == "Other folder…")
            .await?;
        self.stage(cx, NewSessionStage::FolderPage).await?;
        self.capture(cx, "folder")?;
        self.click_text(cx, "the second folder suggestion", |text| {
            text == FOLDER_ROW
        })
        .await?;
        self.until(cx, "the clicked suggestion in the field", |view| {
            Ok((view.new_session_folder_text() == Some(FOLDER_TEXT)).then_some(()))
        })
        .await?;
        // A request sent by the click would show here as a pending step.
        self.timer.timer(QUIET_PERIOD / 3).await;
        self.handle.read_with(cx, |view, _| -> Result<()> {
            ensure!(
                view.new_session_stage() == Some(NewSessionStage::FolderPage),
                "the click left the folder page for {:?}",
                view.new_session_stage()
            );
            ensure!(
                view.new_session_folder_text() == Some(FOLDER_TEXT),
                "the field changed to {:?}",
                view.new_session_folder_text()
            );
            ensure!(view.is_new_session_idle(), "the click sent a request");
            Ok(())
        })??;
        self.capture(cx, "folder-filled")?;
        Ok(format!(
            "a click on the {FOLDER_ROW:?} suggestion filled the field with {FOLDER_TEXT:?}, \
             stayed on the folder page, and sent nothing"
        ))
    }

    /// Waits for a connected window with a coherent snapshot and surface. A
    /// window that is not frontmost publishes its geometry only when drawn, so
    /// each poll focuses and draws it first, as the shared smoke does.
    async fn ready(&self, cx: &mut AsyncApp) -> Result<Ready> {
        let deadline = Instant::now() + DEADLINE;
        loop {
            AnyWindowHandle::from(self.handle).update(cx, |root, window, cx| -> Result<()> {
                let view = root
                    .downcast::<HerdrWindow>()
                    .map_err(|_| anyhow!("unexpected window root"))?;
                window.focus(&view.read(cx).focus.clone(), cx);
                window.refresh();
                window.draw(cx).clear(cx);
                Ok(())
            })??;
            if let Some(ready) = self.handle.read_with(cx, |view, _| ready_state(view))?? {
                return Ok(ready);
            }
            if Instant::now() >= deadline {
                let state = self
                    .handle
                    .read_with(cx, |view, _| connection_state(view))?;
                bail!(
                    "timed out after {DEADLINE:?} waiting for a connected window with a \
                     surface ({state})"
                );
            }
            self.timer.timer(POLL).await;
        }
    }

    fn workspace_ids(&self, cx: &mut AsyncApp) -> Result<Vec<String>> {
        self.handle.read_with(cx, |view, _| {
            view.live
                .snapshot
                .iter()
                .flat_map(|snapshot| {
                    snapshot
                        .workspaces
                        .iter()
                        .map(|workspace| workspace.workspace_id.clone())
                })
                .collect()
        })
    }

    /// Saves the page the footer `new` button opened before the new session
    /// dialog: New Workspace, opened by its shortcut. It is the "before"
    /// screenshot, so the walk skips it, and its key press, unless a capture
    /// folder is named.
    async fn capture_old_footer_page(&self, cx: &mut AsyncApp) -> Result<()> {
        if std::env::var_os(CAPTURE_VARIABLE).is_none() {
            return Ok(());
        }
        // Through the window, not the view: the key handler updates the view.
        AnyWindowHandle::from(self.handle).update(cx, |_, window, cx| -> Result<()> {
            ensure!(
                window.dispatch_keystroke(Keystroke::parse("cmd-shift-n")?, cx),
                "cmd-shift-n was not handled"
            );
            Ok(())
        })??;
        self.until(cx, "the New Workspace page", |view| {
            Ok((view.menu.page == Some(menu::Page::NewWorkspace)).then_some(()))
        })
        .await?;
        self.capture(cx, "before-new-workspace")?;
        self.handle
            .update(cx, |view, window, cx| view.dismiss_menu(window, cx))?;
        self.until(cx, "the New Workspace page to close", |view| {
            Ok(view.menu.page.is_none().then_some(()))
        })
        .await
    }

    /// Clicks the footer's `new` button and waits for the project list.
    async fn open_dialog(&self, cx: &mut AsyncApp) -> Result<()> {
        self.click_text(cx, "the footer new button", |text| text == "new")
            .await?;
        self.stage(cx, NewSessionStage::Projects).await?;
        self.capture(cx, "projects")
    }

    async fn stage(&self, cx: &mut AsyncApp, stage: NewSessionStage) -> Result<()> {
        self.until(cx, &format!("the {stage:?} page"), |view| {
            Ok((view.new_session_stage() == Some(stage)).then_some(()))
        })
        .await
    }

    /// Polls `check` until it returns a value, an error, or `DEADLINE` passes.
    async fn until<T>(
        &self,
        cx: &mut AsyncApp,
        what: &str,
        mut check: impl FnMut(&HerdrWindow) -> Result<Option<T>>,
    ) -> Result<T> {
        let deadline = Instant::now() + DEADLINE;
        loop {
            if let Some(found) = self.handle.read_with(cx, |view, _| check(view))?? {
                return Ok(found);
            }
            if Instant::now() >= deadline {
                let state = self
                    .handle
                    .read_with(cx, |view, _| connection_state(view))?;
                bail!("timed out after {DEADLINE:?} waiting for {what} ({state})");
            }
            self.timer.timer(POLL).await;
        }
    }

    /// Draws a frame and returns the first painted text `wanted` accepts,
    /// with where it was painted.
    fn find_painted(
        &self,
        cx: &mut AsyncApp,
        wanted: &impl Fn(&str) -> bool,
    ) -> Result<Option<(String, Bounds<Pixels>)>> {
        // Through the window, not the view: drawing updates the view itself.
        AnyWindowHandle::from(self.handle).update(cx, |_, window, cx| -> Result<_> {
            window.refresh();
            cx.default_global::<PaintedProbes>().0.clear();
            window.draw(cx).clear(cx);
            let probes = cx.default_global::<PaintedProbes>();
            probes.check()?;
            let Some((text, probe)) = probes.0.iter().find(|(text, _)| wanted(text)) else {
                return Ok(None);
            };
            ensure!(
                !probe.clipped && probe.glyph_text == *text,
                "{text:?} painted cut or clipped: {probe:?}"
            );
            Ok(Some((text.clone(), probe.bounds)))
        })?
    }

    /// The texts of the last drawn frame, for a timeout message.
    fn painted_texts(&self, cx: &mut AsyncApp) -> Result<Vec<String>> {
        AnyWindowHandle::from(self.handle).update(cx, |_, _, cx| {
            cx.default_global::<PaintedProbes>()
                .0
                .keys()
                .cloned()
                .collect()
        })
    }

    /// Waits until `wanted` accepts some painted text and returns that text.
    async fn painted_text(
        &self,
        cx: &mut AsyncApp,
        what: &str,
        wanted: impl Fn(&str) -> bool,
    ) -> Result<String> {
        let deadline = Instant::now() + DEADLINE;
        loop {
            if let Some((text, _)) = self.find_painted(cx, &wanted)? {
                return Ok(text);
            }
            if Instant::now() >= deadline {
                bail!(
                    "timed out after {DEADLINE:?} waiting for {what} to be painted; painted: {:?}",
                    self.painted_texts(cx)?
                );
            }
            self.timer.timer(POLL).await;
        }
    }

    /// Waits for text `wanted` accepts to be painted, then clicks its center.
    async fn click_text(
        &self,
        cx: &mut AsyncApp,
        what: &str,
        wanted: impl Fn(&str) -> bool,
    ) -> Result<()> {
        let deadline = Instant::now() + DEADLINE;
        let bounds = loop {
            if let Some((_, bounds)) = self.find_painted(cx, &wanted)? {
                break bounds;
            }
            if Instant::now() >= deadline {
                bail!(
                    "timed out after {DEADLINE:?} waiting for {what} to be painted; painted: {:?}",
                    self.painted_texts(cx)?
                );
            }
            self.timer.timer(POLL).await;
        };
        let center = bounds.center();
        self.target.click(center.x.to_f64(), center.y.to_f64())
    }

    /// Saves a PNG of the window under `HERDR_TEST_NEW_SESSION_CAPTURE` when
    /// that is set and the build can render images; otherwise does nothing.
    #[cfg(feature = "mockup")]
    fn capture(&self, cx: &mut AsyncApp, name: &str) -> Result<()> {
        let Some(directory) = std::env::var_os(CAPTURE_VARIABLE) else {
            return Ok(());
        };
        let path = std::path::PathBuf::from(directory).join(format!("new-session-{name}.png"));
        ensure!(path.is_absolute(), "{CAPTURE_VARIABLE} must be absolute");
        let image = AnyWindowHandle::from(self.handle).update(cx, |_, window, cx| {
            window.draw(cx).clear(cx);
            window.render_to_image()
        })??;
        image
            .save_with_format(&path, image::ImageFormat::Png)
            .with_context(|| format!("saving {}", path.display()))?;
        eprintln!("NEW SESSION capture: {}", path.display());
        Ok(())
    }

    #[cfg(not(feature = "mockup"))]
    fn capture(&self, _: &mut AsyncApp, _: &str) -> Result<()> {
        let _ = CAPTURE_VARIABLE;
        Ok(())
    }
}

/// The focused workspace and the text rows of its coherent surface, or `None`
/// while the snapshot and surface describe different projections.
fn shell_rows(view: &HerdrWindow) -> Option<(String, Vec<String>)> {
    let (snapshot, surface) = (view.live.snapshot.as_ref()?, view.live.surface.as_ref()?);
    if snapshot.boot_id != surface.boot_id || snapshot.revision != surface.projection_revision {
        return None;
    }
    let rows = surface
        .frame
        .cells
        .chunks(usize::from(surface.frame.width).max(1))
        .map(|row| row.iter().map(|cell| cell.symbol.as_str()).collect())
        .collect();
    Some((snapshot.focused_workspace_id.clone()?, rows))
}

/// The daemon facts the walk needs, once the window shows a coherent snapshot
/// and surface. The sandbox daemon starts with no workspace, so the snapshot
/// may list none.
fn ready_state(view: &HerdrWindow) -> Result<Option<Ready>> {
    if let Some(error) = view.local_error.as_deref().or(view.live.error.as_deref()) {
        bail!("window error: {error:.240}");
    }
    let (Some(snapshot), Some(surface)) = (&view.live.snapshot, &view.live.surface) else {
        return Ok(None);
    };
    if !view.live.status.is_connected()
        || snapshot.boot_id != surface.boot_id
        || snapshot.revision != surface.projection_revision
    {
        return Ok(None);
    }
    Ok(Some(Ready {
        boot: snapshot.boot_id.clone(),
        target: view.endpoints[view.selected_endpoint]
            .connection
            .target
            .clone(),
        options: view.options,
    }))
}

/// What a timeout message says about the window.
fn connection_state(view: &HerdrWindow) -> String {
    format!(
        "status {:.160}, snapshot {:?}, surface {:?}, dialog {:?}, local error {:?}, \
         live error {:?}",
        view.live.status,
        view.live
            .snapshot
            .as_ref()
            .map(|snapshot| (snapshot.revision, snapshot.workspaces.len())),
        view.live
            .surface
            .as_ref()
            .map(|surface| surface.projection_revision),
        view.new_session_stage(),
        view.local_error,
        view.live.error
    )
}

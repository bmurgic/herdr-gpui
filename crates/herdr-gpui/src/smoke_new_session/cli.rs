//! The `cli` walk: steps 1, 3, and 4 of qa.md's "Open the new-session dialog
//! from the palette and the command line", run with the real `herdr-gpui`
//! command line. The harness runs the executable it built as a child, in the
//! sandbox's own environment with no Herdr pane variables, so the command
//! acts in the frontmost window as `docs/cli.md` says a command outside
//! Herdr does. Step 2, the palette, is in the `agents` walk.
//!
//! Native test modes serve no control socket, so a command from a user's
//! agent can never reach a test window. This walk alone serves one, and only
//! after checking that its path lies inside the sandbox.
use super::{
    fixtures::{DEMO, NOTES, OTHER_FOLDER, env_path},
    *,
};
use serde::Deserialize;

/// Exit statuses from `docs/cli.md`'s table.
const EXIT_DONE: i32 = 0;
const EXIT_DIALOG_OPEN: i32 = 6;
/// The refusal the control handler gives while a dialog holds the window.
const DIALOG_OPEN: &str = "A dialog is open in the window";

/// How a `herdr-gpui` child the harness ran ended, as the harness answers.
#[derive(Debug, Deserialize)]
struct CliRun {
    /// `None` when a signal ended it.
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Driver {
    pub(super) async fn cli(&self, cx: &mut AsyncApp) -> Result<String> {
        let fixtures = self.fixtures(cx, false).await?;
        let socket = self.serve_control(cx)?;

        // Step 1: `new_session` is listed with its label and no keys.
        let listed = self.run_cli(cx, &["keybinds", "--json"]).await?;
        ensure!(
            listed.code == Some(EXIT_DONE),
            "keybinds --json ended {listed:?}"
        );
        let listing: serde_json::Value = serde_json::from_str(&listed.stdout)
            .with_context(|| format!("keybinds --json printed {:?}", listed.stdout))?;
        let bindings = listing["bindings"]
            .as_array()
            .with_context(|| format!("keybinds --json has no bindings array: {listing}"))?;
        let entry = bindings
            .iter()
            .find(|binding| binding["name"] == "new_session")
            .with_context(|| format!("keybinds --json lists no new_session: {listing}"))?;
        ensure!(
            entry["label"] == "New Session" && entry["keys"] == serde_json::json!([]),
            "keybinds --json lists new_session as {entry}, not label \"New Session\" and keys []"
        );

        // Step 3: with no dialog open, the command opens the project page.
        ensure!(
            self.handle
                .read_with(cx, |view, _| view.menu.page.is_none())?,
            "a dialog is open before the command runs"
        );
        let opened = self.run_cli(cx, &["run", "new_session"]).await?;
        ensure!(
            opened.code == Some(EXIT_DONE),
            "run new_session with no dialog open ended {opened:?}"
        );
        self.stage(cx, NewSessionStage::Projects).await?;
        self.expect_projects(
            cx,
            &[DEMO, NOTES, OTHER_FOLDER],
            DEMO,
            "from the command line",
        )?;

        // Step 4: over New Workspace, the command is refused and opens nothing.
        self.press(cx, "escape")?;
        self.dialog_closed(cx).await?;
        self.press(cx, "cmd-shift-n")?;
        self.new_workspace_page(cx, "the New Workspace key").await?;
        let refused = self.run_cli(cx, &["run", "new_session"]).await?;
        ensure!(
            refused.code == Some(EXIT_DIALOG_OPEN) && refused.stderr.contains(DIALOG_OPEN),
            "run new_session over New Workspace ended {refused:?}, not exit \
             {EXIT_DIALOG_OPEN} with {DIALOG_OPEN:?}"
        );
        let deadline = Instant::now() + QUIET_PERIOD;
        while Instant::now() < deadline {
            self.handle.read_with(cx, |view, _| -> Result<()> {
                ensure!(
                    view.new_session_stage().is_none()
                        && view.menu.page == Some(menu::Page::NewWorkspace),
                    "after the refused command the window shows {:?}, stage {:?}",
                    view.menu.page,
                    view.new_session_stage()
                );
                Ok(())
            })??;
            self.timer.timer(POLL).await;
        }
        self.press(cx, "escape")?;
        self.expect_no_new_workspace(cx, &fixtures.known, "the walk")
            .await?;
        Ok(format!(
            "keybinds --json listed new_session as \"New Session\" with no keys; run \
             new_session exited {EXIT_DONE} and opened {DEMO}, {NOTES}, {OTHER_FOLDER}; over New \
             Workspace it exited {EXIT_DIALOG_OPEN} ({:?}) and New Workspace stayed open; control \
             socket {}",
            refused.stderr.trim(),
            socket.display()
        ))
    }

    /// Serves the app's control socket, after checking that it is inside the
    /// sandbox, and returns its path.
    fn serve_control(&self, cx: &mut AsyncApp) -> Result<PathBuf> {
        let root = env_path(SANDBOX_VARIABLE)?;
        let state = crate::preferences::state_dir().context("no state folder")?;
        ensure!(
            state.starts_with(&root),
            "the control socket's folder {state:?} is outside the sandbox {root:?}"
        );
        cx.update(crate::control::install);
        // `install` only logs a socket it could not bind.
        let socket = state.join("control.sock");
        ensure!(socket.exists(), "no control socket at {socket:?}");
        Ok(socket)
    }

    /// Asks the harness to run `herdr-gpui` with `args` and waits for it to
    /// end.
    async fn run_cli(&self, cx: &mut AsyncApp, args: &[&str]) -> Result<CliRun> {
        let shown = args.join(" ");
        let request = std::iter::once("cli")
            .chain(args.iter().copied())
            .collect::<Vec<_>>()
            .join("\n");
        let answer = self
            .ask_harness(cx, &request, &format!("run herdr-gpui {shown}"))
            .await?;
        let run: CliRun = serde_json::from_str(&answer)
            .with_context(|| format!("the harness answered {answer:?}"))?;
        eprintln!(
            "NEW SESSION cli {shown}: exit {:?}, stderr {:?}",
            run.code, run.stderr
        );
        Ok(run)
    }
}

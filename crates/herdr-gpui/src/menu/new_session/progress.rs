//! Start progress: the lines and icon that show a request is out, and the
//! rows of an error message.

use super::Step;
use crate::{
    HerdrWindow,
    sidebar::{truncate_end, wrap_two_lines},
};
use gpui::Context;
use std::time::Duration;
use unicode_width::UnicodeWidthStr;

/// How long the loading icon stays on one of its frames.
const FRAME_INTERVAL: Duration = Duration::from_millis(100);

/// How many frames the icon takes to turn once.
const FRAMES_PER_TURN: u8 = 10;

impl Step {
    /// The dim line that names the request this step has out, or nothing for
    /// a step that has no line.
    pub(super) fn label(self) -> Option<&'static str> {
        match self {
            Self::Checking => Some("Checking folder…"),
            Self::Creating => Some("Creating…"),
            Self::Typing => Some("Typing the command…"),
            Self::Focusing => Some("Opening…"),
            Self::Idle | Self::Waiting => None,
        }
    }

    /// Whether the `start` button shows its loading icon turning: while the
    /// start has a request out.
    pub(super) fn is_spinning(self) -> bool {
        matches!(self, Self::Creating | Self::Typing | Self::Focusing)
    }
}

/// How far the loading icon has turned at `frame`, as a fraction of a turn.
pub(super) fn spinner_turn(frame: u8) -> f32 {
    f32::from(frame) / f32::from(FRAMES_PER_TURN)
}

impl HerdrWindow {
    /// Starts the icon's clock at its first frame when a spinning step has
    /// just begun, so the first frame lasts a whole interval.
    pub(super) fn restart_spinner(&mut self, cx: &mut Context<Self>) {
        let now = cx.background_executor().now();
        if let Some(dialog) = self.menu.new_session.as_mut() {
            dialog.spinner_frame = 0;
            dialog.last_frame_at = Some(now);
        }
    }

    /// Moves the icon on by the frames the clock has passed since it last
    /// moved, and asks for a redraw only when it moved. Called from every
    /// window update, so a dialog that spins nothing does no work.
    pub(super) fn advance_spinner(&mut self, cx: &mut Context<Self>) {
        let now = cx.background_executor().now();
        let Some(dialog) = self.menu.new_session.as_mut() else {
            return;
        };
        if !dialog.step.is_spinning() {
            dialog.spinner_frame = 0;
            dialog.last_frame_at = None;
            return;
        }
        let last_frame_at = *dialog.last_frame_at.get_or_insert(now);
        let passed_frames =
            now.saturating_duration_since(last_frame_at).as_millis() / FRAME_INTERVAL.as_millis();
        let Ok(passed_frames) = u32::try_from(passed_frames) else {
            return;
        };
        if passed_frames == 0 {
            return;
        }
        let frame = (u32::from(dialog.spinner_frame) + passed_frames) % u32::from(FRAMES_PER_TURN);
        // Below `FRAMES_PER_TURN`, so the cast keeps every bit.
        dialog.spinner_frame = frame as u8;
        dialog.last_frame_at = Some(last_frame_at + FRAME_INTERVAL * passed_frames);
        cx.notify();
    }
}

/// The rows a failure message takes: at most `max_rows`, each at most
/// `columns` wide. A daemon error can have several lines, such as git's
/// progress line before its error and its hints after it. They stay apart,
/// never joined, and the line that holds the reason always shows, wrapped onto
/// two rows or cut at the end on one. A line before it shows only when it
/// fits whole on a row left free. Lines after it are left out. A port of
/// myherdr's `status_rows`.
pub(super) fn status_rows(message: &str, columns: usize, max_rows: usize) -> Vec<String> {
    let lines: Vec<&str> = message
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    let Some(reason_index) = reason_line_index(&lines) else {
        return Vec::new();
    };
    let mut rows = if max_rows >= 2 {
        let (first, rest) = wrap_two_lines(lines[reason_index], columns);
        if rest.is_empty() {
            vec![first]
        } else {
            vec![first, rest]
        }
    } else {
        vec![truncate_end(lines[reason_index], columns)]
    };
    for line in lines[..reason_index].iter().rev() {
        if rows.len() >= max_rows || line.width() > columns {
            break;
        }
        rows.insert(0, (*line).to_owned());
    }
    rows
}

/// The index of the line that holds a message's reason: the last git error
/// line (`fatal:` or `error:`), else the last line that is not a git hint,
/// else the last line. `None` when there are no lines.
fn reason_line_index(lines: &[&str]) -> Option<usize> {
    let is_error = |line: &&str| line.starts_with("fatal:") || line.starts_with("error:");
    let is_hint = |line: &&str| line.starts_with("hint:");
    lines
        .iter()
        .rposition(is_error)
        .or_else(|| lines.iter().rposition(|line| !is_hint(line)))
        .or_else(|| lines.len().checked_sub(1))
}

//! The `Other folder…` page: a folder field that starts as `~/`, six
//! suggestions under it, and Enter, which checks the text as New Workspace's
//! directory step does before it asks the daemon whether the folder is a git
//! repository (`replies.rs`).
//!
//! The project page's filter stays parked in `NewSession::filter` while the
//! page shows, and Esc brings it back with the highlight it had. The
//! suggestions are `FolderSuggestions`, shared with New Workspace.

use super::{Step, outage::DialogError};
use crate::{
    HerdrWindow,
    dialog_input::DialogInput,
    menu::new_workspace::{DirectoryRefusal, FolderSuggestions, request_cwd},
};
use gpui::{Context, KeyDownEvent, Window};
use std::path::PathBuf;

/// The text the field starts with, as myherdr's does.
const FOLDER_FIELD_START: &str = "~/";

/// The folder page's state while it shows.
pub(super) struct FolderPage {
    suggestions: FolderSuggestions,
    /// The trimmed text whose folder is being checked on the background
    /// executor. Enter is ignored until the check ends.
    existence_check: Option<String>,
}

impl FolderPage {
    pub(super) fn retarget(&mut self, text: &str) -> Option<PathBuf> {
        self.suggestions.retarget(text)
    }

    pub(super) fn suggestions(&self) -> &FolderSuggestions {
        &self.suggestions
    }
}

impl HerdrWindow {
    /// Opens the folder page: the filter is parked and the field holds `~/`
    /// with the cursor at the end, so typing adds to it.
    pub(super) fn open_new_session_folder(&mut self, cx: &mut Context<Self>) {
        let local = self.new_workspace_is_local();
        let Some(dialog) = self.menu.new_session.as_mut() else {
            return;
        };
        let mut suggestions = FolderSuggestions::new(local, Some(dialog.home.clone()));
        let listing = suggestions.retarget(FOLDER_FIELD_START);
        let mut field = DialogInput::new(FOLDER_FIELD_START.into());
        field.selection = field.text.len()..field.text.len();
        dialog.filter = self.menu.input.replace(field);
        dialog.error = None;
        dialog.folder = Some(FolderPage {
            suggestions,
            existence_check: None,
        });
        self.discard_composition(cx);
        if let Some(listing) = listing {
            self.list_new_session_folder(listing, cx);
        }
        cx.notify();
    }

    /// Esc: back to the project page with the filter text and highlight it
    /// had.
    fn leave_new_session_folder(&mut self, cx: &mut Context<Self>) {
        let Some(dialog) = self.menu.new_session.as_mut() else {
            return;
        };
        if dialog.folder.take().is_none() {
            return;
        }
        dialog.error = None;
        self.menu.input = dialog.filter.take();
        self.discard_composition(cx);
        cx.notify();
    }

    /// Lists `folder` off the UI thread and applies the result if the field
    /// still points at it.
    pub(super) fn list_new_session_folder(&mut self, folder: PathBuf, cx: &mut Context<Self>) {
        self.list_folder_in_background(folder, cx, |this, folder, listed, cx| {
            let (Some(page), Some(input)) = (
                this.menu
                    .new_session
                    .as_mut()
                    .and_then(|dialog| dialog.folder.as_mut()),
                this.menu.input.as_ref(),
            ) else {
                return;
            };
            if page.suggestions.finish_listing(folder, listed, &input.text) {
                cx.notify();
            }
        });
    }

    /// Keys on the folder page: Up and Down move the suggestion, Tab fills
    /// it, Enter checks the folder, and Esc leaves. Returns whether the key
    /// was used.
    pub(super) fn folder_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(page) = self
            .menu
            .new_session
            .as_mut()
            .and_then(|dialog| dialog.folder.as_mut())
        else {
            return false;
        };
        match event.keystroke.key.as_str() {
            "up" => page.suggestions.move_selection(-1),
            "down" => page.suggestions.move_selection(1),
            "tab" => self.complete_new_session_folder(cx),
            "enter" => self.submit_new_session_folder(cx),
            "escape" => self.leave_new_session_folder(cx),
            _ => return false,
        }
        cx.stop_propagation();
        window.prevent_default();
        cx.notify();
        true
    }

    /// A left click on the suggestion at `index`: select it and fill the
    /// field as Tab does. Sends nothing.
    pub(super) fn click_new_session_suggestion(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.new_session_locked() {
            return;
        }
        let Some(page) = self
            .menu
            .new_session
            .as_mut()
            .and_then(|dialog| dialog.folder.as_mut())
        else {
            return;
        };
        page.suggestions.select(index);
        self.complete_new_session_folder(cx);
        cx.notify();
    }

    /// Puts the selected suggestion in the field, without a trailing `/`.
    fn complete_new_session_folder(&mut self, cx: &mut Context<Self>) {
        let Some(completion) = self
            .menu
            .new_session
            .as_ref()
            .and_then(|dialog| dialog.folder.as_ref())
            .and_then(|page| page.suggestions.completion())
        else {
            return;
        };
        self.edit_menu_input(cx, |input, _| {
            input.selection = 0..input.text.len();
            input.replace(None, &completion, false, None);
            true
        });
    }

    /// Enter: refuses the text, or checks it. A folder on this computer must
    /// exist, which is read off the UI thread. A refusal sends nothing.
    fn submit_new_session_folder(&mut self, cx: &mut Context<Self>) {
        let snapshot_v2 = self.live.snapshot_v2;
        let (Some(dialog), Some(input)) =
            (self.menu.new_session.as_mut(), self.menu.input.as_ref())
        else {
            return;
        };
        let Some(page) = dialog.folder.as_mut() else {
            return;
        };
        if page.existence_check.is_some() {
            return;
        }
        let text = input.text.trim().to_owned();
        if let Some(refusal) = page.suggestions.refusal_before_check(&text, snapshot_v2) {
            dialog.error = Some(DialogError::Message(refusal.message().to_owned()));
            return;
        }
        if !page.suggestions.is_local() {
            self.check_folder(&text, None, cx);
            return;
        }
        let Some(path) = page.suggestions.expand(&text) else {
            dialog.error = Some(DialogError::Message(
                DirectoryRefusal::Missing.message().to_owned(),
            ));
            return;
        };
        page.existence_check = Some(text.clone());
        let existence = cx.background_executor().spawn(async move { path.is_dir() });
        cx.spawn(async move |this, cx| {
            let exists = existence.await;
            let _ = this.update(cx, |this, cx| {
                this.finish_new_session_existence_check(&text, exists, cx);
            });
        })
        .detach();
    }

    /// Applies the existence check of `text`, unless the page closed or the
    /// text changed while the folder was being checked.
    fn finish_new_session_existence_check(
        &mut self,
        text: &str,
        exists: bool,
        cx: &mut Context<Self>,
    ) {
        let (Some(dialog), Some(input)) =
            (self.menu.new_session.as_mut(), self.menu.input.as_ref())
        else {
            return;
        };
        let Some(page) = dialog.folder.as_mut() else {
            return;
        };
        if page.existence_check.take().as_deref() != Some(text) || input.text.trim() != text {
            return;
        }
        if dialog.step != Step::Idle {
            return;
        }
        if exists {
            self.check_folder(text, None, cx);
        } else {
            dialog.error = Some(DialogError::Message(
                DirectoryRefusal::Missing.message().to_owned(),
            ));
            cx.notify();
        }
    }

    /// The `cwd` the daemon is sent for `text`: the trimmed text on a v2
    /// session, the expanded path on a v1 session. `None` when that needs a
    /// home that JSON cannot carry.
    pub(super) fn new_session_request_cwd(&self, text: &str) -> Option<String> {
        let home = &self.menu.new_session.as_ref()?.home;
        request_cwd(text, self.live.snapshot_v2, Some(home))
    }
}

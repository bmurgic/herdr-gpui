//! What the native walk (`smoke_new_session.rs`) reads of the dialog: the page
//! it shows and the few fields its clicks must change. The walk waits for
//! these after each click, so it never sleeps for a fixed time.

use super::Step;
use crate::HerdrWindow;

/// The page the dialog shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NewSessionStage {
    Projects,
    FolderPage,
    Confirm,
    /// The setup checklist, with a hook row showing.
    Checklist,
}

impl HerdrWindow {
    /// The page the dialog shows, or `None` while it is closed.
    pub(crate) fn new_session_stage(&self) -> Option<NewSessionStage> {
        let dialog = self.menu.new_session.as_ref()?;
        Some(if dialog.folder.is_some() {
            NewSessionStage::FolderPage
        } else if self.new_session_checklist().is_some() {
            NewSessionStage::Checklist
        } else if dialog.confirm.is_some() {
            NewSessionStage::Confirm
        } else {
            NewSessionStage::Projects
        })
    }

    /// Whether the dialog waits for no request, which a click on a suggestion
    /// must leave true.
    pub(crate) fn is_new_session_idle(&self) -> bool {
        self.menu
            .new_session
            .as_ref()
            .is_some_and(|dialog| dialog.step == Step::Idle && dialog.pending.is_none())
    }

    /// The text of the folder page's field.
    pub(crate) fn new_session_folder_text(&self) -> Option<&str> {
        self.menu.new_session.as_ref()?.folder.as_ref()?;
        Some(self.menu.input.as_ref()?.text.as_str())
    }
}

//! The folder suggestions both folder fields share: where the typed text
//! points, the folders found there, and which of them is selected. New
//! Workspace's directory step and the new-session dialog's `Other folder…`
//! page each hold one, so the ranking, the listing key, Tab completion, and
//! the Up and Down rule exist once.
//!
//! A listing reads the disk and blocks, so the owner runs it on the
//! background executor and hands the result to `finish_listing`, which drops
//! it unless the text still points at the folder that was listed.

use super::suggestions::{
    DirectoryRefusal, MAX_FOLDER_ENTRIES, check_form, expand_tilde, list_folders, ranked_matches,
};
use crate::HerdrWindow;
use gpui::Context;
use std::{
    io,
    path::{Path, PathBuf},
};

/// The folder suggestions of one text field.
pub(in crate::menu) struct FolderSuggestions {
    /// Whether the field's host is this computer, the only host whose disk
    /// the client may read.
    local: bool,
    /// What a leading `~` expands to, read once when the field opens.
    home: Option<PathBuf>,
    /// The text through its last `/`, which Tab completion puts a folder
    /// name after. `None` when there is nothing to complete.
    pub(super) parent: Option<String>,
    /// The absolute folder `parent` names, which `children` came from or is
    /// being listed from.
    folder: Option<PathBuf>,
    children: Vec<String>,
    pub(super) matches: Vec<String>,
    pub(super) selected: usize,
    /// Why the folder could not be read.
    listing_error: Option<String>,
}

impl FolderSuggestions {
    pub(in crate::menu) fn new(local: bool, home: Option<PathBuf>) -> Self {
        Self {
            local,
            home,
            parent: None,
            folder: None,
            children: Vec::new(),
            matches: Vec::new(),
            selected: 0,
            listing_error: None,
        }
    }

    pub(in crate::menu) fn is_local(&self) -> bool {
        self.local
    }

    pub(in crate::menu) fn home(&self) -> Option<&Path> {
        self.home.as_deref()
    }

    /// Why the folder could not be read.
    pub(in crate::menu) fn listing_error(&self) -> Option<&str> {
        self.listing_error.as_deref()
    }

    /// The suggestion rows shown, at most `rows`, scrolled so the selected one
    /// is among them, each with its index in `matches`.
    pub(in crate::menu) fn visible(&self, rows: usize) -> impl Iterator<Item = (usize, &str)> {
        self.matches
            .iter()
            .enumerate()
            .skip(self.visible_start(rows))
            .take(rows)
            .map(|(index, name)| (index, name.as_str()))
    }

    /// Why `text`, already trimmed, cannot be sent. A folder on this computer
    /// is checked afterwards, and a daemon checks its own.
    pub(in crate::menu) fn refusal_before_check(
        &self,
        text: &str,
        snapshot_v2: bool,
    ) -> Option<DirectoryRefusal> {
        if let Err(refusal) = check_form(text) {
            return Some(refusal);
        }
        // Without the v2 codec the daemon neither expands `~` nor checks the
        // folder, and only this computer's home is known here.
        (!self.local && !snapshot_v2 && text.starts_with('~')).then_some(DirectoryRefusal::Tilde)
    }

    /// `text` with a leading `~` expanded to the home folder. `None` when
    /// that needs a home and there is none.
    pub(in crate::menu) fn expand(&self, text: &str) -> Option<PathBuf> {
        expand_tilde(text, self.home.as_deref())
    }

    /// Whether `index`, an index in `matches`, is the selected suggestion.
    pub(in crate::menu) fn is_selected(&self, index: usize) -> bool {
        index == self.selected
    }

    /// Forgets the listing, for a text with no parent folder to list.
    pub(in crate::menu) fn clear_listing(&mut self) {
        self.parent = None;
        self.folder = None;
        self.children.clear();
        self.matches.clear();
        self.selected = 0;
        self.listing_error = None;
    }

    /// Follows an edit of the text. A change of only the part after the last
    /// `/` ranks the folders already listed again. A change of the parent
    /// forgets them, so Tab and the arrow keys never act on another parent's
    /// folders, and returns the folder to list.
    pub(in crate::menu) fn retarget(&mut self, text: &str) -> Option<PathBuf> {
        let text = if text.is_empty() { "~/" } else { text };
        let Some((parent, _)) = text.rsplit_once('/').filter(|_| self.local) else {
            self.clear_listing();
            return None;
        };
        let parent = format!("{parent}/");
        let folder = expand_tilde(&parent, self.home.as_deref()).filter(|path| path.is_absolute());
        let changed = self.folder != folder;
        if changed {
            self.children.clear();
            self.listing_error = None;
            self.folder = folder.clone();
        }
        self.parent = Some(parent);
        self.matches = ranked_matches(&self.children, query(text));
        self.selected = 0;
        folder.filter(|_| changed)
    }

    /// Takes the result of listing `folder`. A result for a folder the text
    /// has since moved away from is dropped. Returns whether anything changed.
    pub(in crate::menu) fn finish_listing(
        &mut self,
        folder: &Path,
        listed: io::Result<Vec<String>>,
        text: &str,
    ) -> bool {
        if self.folder.as_deref() != Some(folder) {
            return false;
        }
        match listed {
            Ok(children) => self.children = children,
            Err(error) => self.listing_error = Some(format!("Cannot read folders: {error}")),
        }
        self.matches = ranked_matches(&self.children, query(text));
        self.selected = 0;
        true
    }

    /// Moves the selection by `delta`, stopping at either end.
    pub(in crate::menu) fn move_selection(&mut self, delta: isize) {
        self.selected = self
            .selected
            .saturating_add_signed(delta)
            .min(self.matches.len().saturating_sub(1));
    }

    /// Selects the suggestion at `index` in `matches`, when there is one.
    pub(in crate::menu) fn select(&mut self, index: usize) {
        if index < self.matches.len() {
            self.selected = index;
        }
    }

    /// The text Tab puts in the field: the parent and the selected folder,
    /// with no trailing `/`, so a second `/` descends on purpose.
    pub(in crate::menu) fn completion(&self) -> Option<String> {
        Some(format!(
            "{}{}",
            self.parent.as_ref()?,
            self.matches.get(self.selected)?
        ))
    }

    /// The first row shown, so the selected one is the last when it is below
    /// the first screenful.
    fn visible_start(&self, rows: usize) -> usize {
        self.selected.saturating_sub(rows.saturating_sub(1))
    }
}

/// The part of `text` after its last `/`, which folder names are ranked by.
fn query(text: &str) -> &str {
    text.rsplit_once('/').map_or("", |(_, query)| query)
}

impl HerdrWindow {
    /// Lists `folder` on the background executor and hands the result to
    /// `apply` on the UI thread, which must check that its field still points
    /// at `folder` (`FolderSuggestions::finish_listing` does).
    pub(in crate::menu) fn list_folder_in_background(
        &mut self,
        folder: PathBuf,
        cx: &mut Context<Self>,
        apply: impl FnOnce(&mut Self, &Path, io::Result<Vec<String>>, &mut Context<Self>) + 'static,
    ) {
        let listing = cx.background_executor().spawn({
            let folder = folder.clone();
            async move { list_folders(&folder, MAX_FOLDER_ENTRIES) }
        });
        cx.spawn(async move |this, cx| {
            let listed = listing.await;
            let _ = this.update(cx, |this, cx| apply(this, &folder, listed, cx));
        })
        .detach();
    }
}

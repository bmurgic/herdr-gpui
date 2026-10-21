use super::*;

/// The command the Agents tab's new dialog types: `claude` unless the file
/// holds a string, which is kept exactly as written.
#[test]
fn a_value_that_is_not_a_string_reads_as_claude_and_spares_its_siblings() -> anyhow::Result<()> {
    for value in ["7", "true", "[\"claude\"]"] {
        let settings = Settings::parse_text(&format!(
            "[ui]\nnew_session_command = {value}\nsession_sidebar = false\n"
        ))?;
        assert_eq!(settings.new_session_command, "claude", "{value}");
        assert!(!settings.session_sidebar, "the sibling key reads: {value}");
    }
    Ok(())
}

#[test]
fn a_missing_key_reads_claude_and_a_string_reads_back_untrimmed() -> anyhow::Result<()> {
    let missing = Settings::parse_text("[ui]\nsession_sidebar = false\ncopy_on_select = false\n")?;
    assert_eq!(missing.new_session_command, "claude");

    for saved in ["", "  ", "codex --yolo"] {
        let settings = Settings::parse_text(&format!("[ui]\nnew_session_command = \"{saved}\"\n"))?;
        assert_eq!(settings.new_session_command, saved);
    }
    Ok(())
}

/// Each value is saved as a TOML string under `[ui]`, beside the keys already
/// there and ahead of the next table, then read back through the parser.
#[cfg(unix)]
#[test]
fn the_edit_saves_the_text_under_ui_and_reads_back_as_written() -> anyhow::Result<()> {
    let original = "[ui]\nsession_sidebar = false\n[experimental]\npane_history = true\n";
    for saved in ["codex", "claude --resume", "", "a\"b", "a\"b\\c"] {
        let temp = tempfile::tempdir()?;
        let path = temp.path().join("config.toml");
        fs::write(&path, original)?;
        Settings::load_path(path.clone())?.save(Edit::NewSessionCommand(saved.into()))?;

        let read_back = Settings::parse_text(&fs::read_to_string(&path)?)?;
        assert_eq!(read_back.new_session_command, saved);
        assert!(!read_back.session_sidebar, "the sibling key survives");
        assert!(read_back.pane_history, "the next table survives");
    }
    Ok(())
}

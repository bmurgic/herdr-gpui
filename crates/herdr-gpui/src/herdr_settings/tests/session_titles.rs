use super::*;

fn assert_defaults(settings: &Settings, what: &str) {
    assert!(settings.session_titles.enabled, "{what}");
    assert_eq!(settings.session_titles.agent, TitleAgent::Claude, "{what}");
}

#[test]
fn absent_section_reads_the_defaults() -> anyhow::Result<()> {
    let settings = parsed("[ui]\n")?;
    assert_defaults(&settings, "absent section");
    assert!(!settings.session_titles_invalid);
    Ok(())
}

#[test]
fn configured_section_reads_its_values() -> anyhow::Result<()> {
    let settings = parsed(
        "[session_titles]\nenabled = false\nagent = \"codex\"\nclaude_path = \"/opt/claude\"\nfuture = 1\n",
    )?;
    assert!(!settings.session_titles.enabled);
    assert_eq!(settings.session_titles.agent, TitleAgent::Codex);
    assert!(!settings.session_titles_invalid);
    Ok(())
}

#[test]
fn invalid_sections_read_the_defaults_and_are_flagged() -> anyhow::Result<()> {
    // Each case differs from the defaults in `enabled` or `agent` before the
    // bad key, so a section read half-way would not look like the defaults.
    for (what, text) in [
        ("not a table", "session_titles = 5\n"),
        (
            "unknown agent",
            "[session_titles]\nenabled = false\nagent = \"grok\"\n",
        ),
        (
            "enabled is a string",
            "[session_titles]\nagent = \"codex\"\nenabled = \"yes\"\n",
        ),
        (
            "claude_path is a number",
            "[session_titles]\nenabled = false\nagent = \"codex\"\nclaude_path = 5\n",
        ),
        (
            "codex_path is a boolean",
            "[session_titles]\nenabled = false\nagent = \"codex\"\ncodex_path = true\n",
        ),
    ] {
        let sibling = format!("{text}[ui]\nsession_sidebar = false\n");
        let settings = parsed(&sibling)?;
        assert_defaults(&settings, what);
        assert!(settings.session_titles_invalid, "{what}");
        assert!(
            !settings.session_sidebar,
            "{what}: a sibling section still reads"
        );
    }
    Ok(())
}

#[test]
fn keep_session_titles_keeps_previous_values_only_for_an_invalid_section() -> anyhow::Result<()> {
    let previous = parsed("[session_titles]\nenabled = false\nagent = \"codex\"\n")?;

    let mut next = parsed("[session_titles]\nagent = \"grok\"\n")?;
    next.keep_session_titles(&previous);
    assert!(!next.session_titles.enabled);
    assert_eq!(next.session_titles.agent, TitleAgent::Codex);

    let mut next = parsed("[session_titles]\nenabled = true\n")?;
    next.keep_session_titles(&previous);
    assert!(next.session_titles.enabled);
    assert_eq!(next.session_titles.agent, TitleAgent::Claude);

    let mut next = parsed("[ui]\n")?;
    next.keep_session_titles(&previous);
    assert_defaults(&next, "absent section");
    Ok(())
}

#[cfg(unix)]
#[test]
fn session_edits_add_their_section_and_keep_other_lines() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let path = temp.path().join("config.toml");
    let original = "# keep\n[theme]\nname = \"catppuccin\"\n";
    fs::write(&path, original)?;
    let settings = Settings::load_path(path.clone())?.save(Edit::SessionSidebar(false))?;
    assert!(!settings.session_sidebar);
    assert!(fs::read_to_string(&path)?.starts_with(original));
    assert!(!Settings::load_path(path.clone())?.session_sidebar);

    let settings = settings.save(Edit::SessionTitlesEnabled(false))?;
    assert!(!settings.session_titles.enabled);
    assert!(!Settings::load_path(path.clone())?.session_titles.enabled);

    let settings = settings.save(Edit::TitleAgent(TitleAgent::Codex))?;
    assert_eq!(settings.session_titles.agent, TitleAgent::Codex);
    let text = fs::read_to_string(&path)?;
    assert!(text.starts_with(original));
    assert!(text.contains("agent = \"codex\"\nmodel = \"\"\n"), "{text}");
    assert_eq!(
        Settings::load_path(path)?.session_titles.agent,
        TitleAgent::Codex
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn session_sidebar_writes_true_and_false() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let path = temp.path().join("config.toml");
    fs::write(&path, "[ui]\nsession_sidebar = false # rows\n")?;
    let settings = Settings::load_path(path.clone())?.save(Edit::SessionSidebar(true))?;
    assert_eq!(
        fs::read_to_string(&path)?,
        "[ui]\nsession_sidebar = true # rows\n"
    );
    settings.save(Edit::SessionSidebar(false))?;
    assert_eq!(
        fs::read_to_string(&path)?,
        "[ui]\nsession_sidebar = false # rows\n"
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn title_agent_clears_the_model_in_one_write() -> anyhow::Result<()> {
    for (before, agent) in [
        (
            "[session_titles]\nagent = \"claude\"\nmodel = \"sonnet\"\n",
            TitleAgent::Codex,
        ),
        ("[session_titles]\nagent = \"claude\"\n", TitleAgent::Codex),
        (
            "[session_titles]\nagent = \"codex\"\nmodel = \"custom-luna\"\n",
            TitleAgent::Claude,
        ),
    ] {
        let temp = tempfile::tempdir()?;
        let path = temp.path().join("config.toml");
        fs::write(&path, before)?;
        Settings::load_path(path.clone())?.save(Edit::TitleAgent(agent))?;
        assert_eq!(
            fs::read_to_string(&path)?,
            format!(
                "[session_titles]\nagent = \"{}\"\nmodel = \"\"\n",
                agent.as_str()
            ),
            "{before}"
        );
    }
    Ok(())
}

#[test]
fn title_model_reads_and_defaults() -> anyhow::Result<()> {
    assert_eq!(parsed("[session_titles]\n")?.session_titles.model, "");
    assert_eq!(parsed("[ui]\n")?.session_titles.model, "");
    assert_eq!(
        parsed("[session_titles]\nmodel = \"sonnet\"\n")?
            .session_titles
            .model,
        "sonnet"
    );

    // The agent and switch differ from the defaults, so a section read
    // half-way would not look like them.
    let invalid = parsed("[session_titles]\nenabled = false\nagent = \"codex\"\nmodel = 5\n")?;
    assert!(invalid.session_titles_invalid);
    assert_eq!(invalid.session_titles, SessionTitles::default());

    assert_eq!(TitleAgent::Claude.default_model(), "haiku");
    assert_eq!(TitleAgent::Codex.default_model(), "gpt-6-luna");
    Ok(())
}

#[cfg(unix)]
#[test]
fn title_model_edit_reads_back_as_written() -> anyhow::Result<()> {
    for (model, line) in [
        ("my-model", "model = \"my-model\"\n"),
        // toml_edit picks the quoting; the text must survive either way.
        ("a\"b\\c", ""),
        ("", "model = \"\"\n"),
    ] {
        let temp = tempfile::tempdir()?;
        let path = temp.path().join("config.toml");
        let original = "# keep\n[session_titles]\nenabled = false # on\nagent = \"codex\"\n";
        fs::write(&path, original)?;
        let saved = Settings::load_path(path.clone())?.save(Edit::TitleModel(model.into()))?;
        assert_eq!(saved.session_titles.model, model);

        let text = fs::read_to_string(&path)?;
        assert!(text.starts_with(original), "{text}");
        assert!(text.contains(line), "{text}");
        let reloaded = Settings::load_path(path)?;
        assert!(!reloaded.session_titles_invalid);
        assert_eq!(reloaded.session_titles.model, model);
        assert_eq!(reloaded.session_titles.agent, TitleAgent::Codex);
    }
    Ok(())
}

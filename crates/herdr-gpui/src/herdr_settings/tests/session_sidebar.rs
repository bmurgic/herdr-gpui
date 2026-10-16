use super::*;

/// `ui.session_sidebar` follows myherdr: session rows are on unless the file
/// says `false`, and a value of the wrong type keeps the default.
#[test]
fn absent_key_turns_session_rows_on() -> anyhow::Result<()> {
    assert!(Settings::parse_text("[ui]\n")?.session_sidebar);
    Ok(())
}

#[test]
fn false_turns_session_rows_off() -> anyhow::Result<()> {
    assert!(!Settings::parse_text("[ui]\nsession_sidebar = false\n")?.session_sidebar);
    Ok(())
}

#[test]
fn true_keeps_session_rows_on() -> anyhow::Result<()> {
    assert!(Settings::parse_text("[ui]\nsession_sidebar = true\n")?.session_sidebar);
    Ok(())
}

#[test]
fn a_wrong_type_falls_back_to_on() -> anyhow::Result<()> {
    let settings =
        Settings::parse_text("[ui]\nsession_sidebar = \"yes\"\ncopy_on_select = false\n")?;
    assert!(settings.session_sidebar);
    assert!(!settings.copy_on_select, "the sibling key still reads");
    Ok(())
}

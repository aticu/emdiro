//! Handles interaction with the user interface.

use crate::Position;

/// Moves the mouse to the specified position.
pub(crate) fn move_mouse(Position { x, y }: Position) -> anyhow::Result<()> {
    if !std::process::Command::new("ydotool")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .arg("mousemove")
        .arg("--absolute")
        .args(["-x", &format!("{x}")])
        .args(["-y", &format!("{y}")])
        .status()?
        .success()
    {
        anyhow::bail!("ydotool mousemove failed");
    }

    Ok(())
}

/// Clicks on the given position on the screen.
pub(crate) fn click(position: Position) -> anyhow::Result<()> {
    move_mouse(position)?;

    if !std::process::Command::new("ydotool")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .arg("click")
        .arg("40")
        .arg("80")
        .status()?
        .success()
    {
        anyhow::bail!("ydotool click failed");
    }

    Ok(())
}

/// Presses the given keys in the given order all at once, then releases them in reverse order.
pub(crate) fn press_keys(keys: &[u32]) -> anyhow::Result<()> {
    if !std::process::Command::new("ydotool")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .arg("key")
        .args(keys.iter().map(|key| format!("{key}:1")))
        .args(keys.iter().rev().map(|key| format!("{key}:0")))
        .status()?
        .success()
    {
        anyhow::bail!("ydotool key failed");
    }

    Ok(())
}

/// Types the given text.
pub(crate) fn r#type(text: &str) -> anyhow::Result<()> {
    if !std::process::Command::new("ydotool")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .arg("type")
        .arg(text)
        .status()?
        .success()
    {
        anyhow::bail!("ydotool type failed");
    }

    Ok(())
}

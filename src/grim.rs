//! Handles taking screenshots through grim.

/// Takes a screenshot of the given screen rectangle.
pub(crate) fn take_screenshot(
    super::Rect {
        x,
        y,
        width,
        height,
    }: super::Rect,
) -> anyhow::Result<Option<image::RgbImage>> {
    let output = std::process::Command::new("grim")
        .args(["-l", "0"])
        .args(["-g", &format!("{x},{y} {width}x{height}")])
        .arg("-")
        .output()?;

    Ok(Some(image::load_from_memory(&output.stdout)?.to_rgb8()))
}

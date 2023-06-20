//! Handles querying screen positions through slurp.

/// Queries the user for a rectangle on the screen.
pub(crate) fn query_rect(point: bool) -> anyhow::Result<Option<super::Rect>> {
    let mut command = std::process::Command::new("slurp");
    if point {
        command.arg("-p");
    }
    let output = command.output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let unexpect_format_err = || anyhow::anyhow!("unexpected slurp format");

    let output = std::str::from_utf8(&output.stdout)?;
    let x = output
        .split(',')
        .next()
        .ok_or_else(unexpect_format_err)?
        .parse()?;
    let y = output
        .split(',')
        .nth(1)
        .ok_or_else(unexpect_format_err)?
        .split(' ')
        .next()
        .ok_or_else(unexpect_format_err)?
        .parse()?;
    let width = output
        .split(' ')
        .nth(1)
        .ok_or_else(unexpect_format_err)?
        .split('x')
        .next()
        .ok_or_else(unexpect_format_err)?
        .parse()?;
    let height = output
        .split(' ')
        .nth(1)
        .ok_or_else(unexpect_format_err)?
        .split('x')
        .nth(1)
        .ok_or_else(unexpect_format_err)?
        .trim_end()
        .parse()?;

    Ok(Some(super::Rect {
        x,
        y,
        width,
        height,
    }))
}

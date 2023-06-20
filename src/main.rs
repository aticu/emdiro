use std::path::PathBuf;

use structopt::StructOpt;

mod command;
mod grim;
mod key_codes;
mod slurp;
mod ydotool;

/// Kills the wrapped child process on drop.
struct KillOnDrop(std::process::Child);

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        self.0.kill().unwrap();
    }
}

/// A position on the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct Position {
    /// The x position.
    pub(crate) x: u32,
    /// The y position.
    pub(crate) y: u32,
}

/// A rectangle with integer coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct Rect {
    /// The lowest x position.
    pub(crate) x: u32,
    /// The lowest y position.
    pub(crate) y: u32,
    /// The width of the rectangle.
    pub(crate) width: u32,
    /// The height of the rectangle.
    pub(crate) height: u32,
}

impl Rect {
    /// Returns the origin of the rectangle.
    pub(crate) fn origin(self) -> Position {
        Position {
            x: self.x,
            y: self.y,
        }
    }

    /// Computes the center of the rectangle.
    pub(crate) fn center(self) -> Position {
        Position {
            x: self.x + self.width / 2,
            y: self.y + self.height / 2,
        }
    }
}

/// Starts the `ydotoold` process.
fn start_ydotoold() -> KillOnDrop {
    KillOnDrop(
        std::process::Command::new("ydotoold")
            .arg("-P")
            .arg("0660")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap(),
    )
}

/// lEt Me Do It foR yOu: simple automation on linux
#[derive(Debug, StructOpt)]
enum Config {
    /// record a chain of commands that can later be replayed
    Record {
        /// the file where the command chain should be stored
        commandfile: PathBuf,
    },
    /// runs a previously recorded chain of commands
    Run {
        /// the file where the command chain is stored
        commandfile: PathBuf,
        /// the number of runs to perform
        #[structopt(long, short, default_value = "1")]
        num_runs: u32,
    },
}

fn main() -> anyhow::Result<()> {
    let config = Config::from_args();

    match config {
        Config::Record { commandfile } => {
            let chain = command::CommandChain::record()?;
            serde_json::to_writer_pretty(std::fs::File::create(commandfile)?, &chain)?;
        }
        Config::Run {
            commandfile,
            num_runs,
        } => {
            let _ydotoold = start_ydotoold();
            let chain: command::CommandChain =
                serde_json::from_reader(std::fs::File::open(commandfile)?)?;

            for i in 0..num_runs {
                println!("Starting run {}/{num_runs}", i + 1);
                chain.execute()?;
            }
        }
    }

    Ok(())
}

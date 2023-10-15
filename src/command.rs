//! Handles construction and execution of scriptable commands.

use std::{path::Path, time::Duration};

use image::RgbImage;

use crate::{grim::take_screenshot, slurp::query_rect, ydotool, Position, Rect};

mod serde_img {
    use base64::Engine;

    /// Serialize an image.
    pub(super) fn serialize<S: serde::ser::Serializer>(
        img: &image::RgbImage,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut img_buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut img_buf, image::ImageOutputFormat::Png)
            .unwrap();

        serializer
            .serialize_str(&base64::engine::general_purpose::STANDARD.encode(img_buf.into_inner()))
    }

    /// Deserialize an image.
    pub(super) fn deserialize<'de, D: serde::de::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<image::RgbImage, D::Error> {
        struct StrVisitor;

        impl<'de> serde::de::Visitor<'de> for StrVisitor {
            type Value = Vec<u8>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a base64 encoded PNG image")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                base64::engine::general_purpose::STANDARD
                    .decode(v)
                    .map_err(|err| E::custom(err))
            }
        }

        let bytes = deserializer.deserialize_str(StrVisitor)?;

        Ok(image::load_from_memory(&bytes).unwrap().to_rgb8())
    }
}

/// A single command in a chain of commands.
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) enum Command {
    /// Waits until an image is present at the given location.
    WaitForImage {
        /// The location on the screen where the image should appear.
        location: Rect,
        /// The image that is being waited for.
        #[serde(with = "serde_img")]
        image: RgbImage,
        /// Whether the image should be clicked after it appears.
        click: bool,
    },
    /// Sleeps for a specified duration.
    Sleep {
        /// The duration of the sleep.
        duration: Duration,
    },
    /// Runs the given shell command in bash.
    Shell {
        /// The shell command to run.
        command: String,
    },
    /// Presses the given keys all at once in the given order.
    PressKeys {
        /// The keys to press.
        keys: Vec<u32>,
    },
    /// Types the given text.
    Type {
        /// The text to type.
        text: String,
    },
    /// Clicks on the given position.
    Click {
        /// The position to click onto.
        position: Position,
    },
    /// Moves the mouse to the given position.
    MouseMove {
        /// The position to click onto.
        position: Position,
    },
}

impl Command {
    /// Constructs a new wait for image command.
    pub(crate) fn wait_for_image(click: bool) -> anyhow::Result<Option<Self>> {
        let Some(location) = query_rect(false)? else { return Ok(None) };

        while !dialoguer::Confirm::new()
            .with_prompt("Is the image presented as it should be?")
            .interact()?
        {}

        let Some(image) = take_screenshot(location)? else { return Ok(None) };

        Ok(Some(Command::WaitForImage {
            location,
            image,
            click,
        }))
    }

    /// Executes the command.
    pub(crate) fn execute(&self) -> anyhow::Result<()> {
        match self {
            Self::WaitForImage {
                location,
                image,
                click,
            } => {
                loop {
                    let Some(curr_image) = take_screenshot(*location)? else { continue };
                    if &curr_image == image {
                        break;
                    }
                }
                if *click {
                    ydotool::click(location.center())?;
                }
            }
            Self::Sleep { duration } => {
                std::thread::sleep(*duration);
            }
            Self::Shell { command } => {
                let status = std::process::Command::new("bash")
                    .args(["-c", command])
                    .status()?;
                if !status.success() {
                    anyhow::bail!("shell command `{command}` exited with status {status}");
                }
            }
            Self::PressKeys { keys } => {
                ydotool::press_keys(keys)?;
            }
            Self::Type { text } => {
                ydotool::r#type(text)?;
            }
            Self::Click { position } => {
                ydotool::click(*position)?;
            }
            Self::MouseMove { position } => {
                ydotool::move_mouse(*position)?;
            }
        }

        Ok(())
    }
}

/// The dialogue options presented to the user.
const OPTIONS: &[&str] = &[
    "wait for image and click",
    "wait for image",
    "click",
    "move mouse",
    "press keys",
    "type text",
    "shell command",
    "sleep",
    "exit run",
];

/// Contains commands that should be executed in a chain.
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct CommandChain {
    /// The commands in the chain.
    commands: Vec<Command>,
}

impl CommandChain {
    /// Records a new chain of commands.
    pub(crate) fn record() -> anyhow::Result<Self> {
        let key_codes = crate::key_codes::KeyCodes::new()?;

        let mut commands = Vec::new();

        loop {
            let command = match OPTIONS[dialoguer::FuzzySelect::new()
                .with_prompt("select your next command")
                .items(OPTIONS)
                .default(0)
                .interact()?]
            {
                option @ ("wait for image and click" | "wait for image") => {
                    Command::wait_for_image(option == "wait for image and click")?
                }
                "click" => query_rect(true)?.map(|rect| Command::Click {
                    position: rect.origin(),
                }),
                "move mouse" => query_rect(true)?.map(|rect| Command::MouseMove {
                    position: rect.origin(),
                }),
                "press keys" => {
                    let mut keys = Vec::new();
                    while let Some(index) = dialoguer::FuzzySelect::new()
                        .with_prompt(
                            "select a key code or press a ESC to finish selecting key codes",
                        )
                        .items(key_codes.codes())
                        .interact_opt()?
                    {
                        if let Some(key) = key_codes.get_num(index) {
                            keys.push(key);
                        }
                    }

                    Some(Command::PressKeys { keys })
                }
                "type text" => {
                    let text = dialoguer::Input::new()
                        .with_prompt("enter the text to type")
                        .interact_text()?;
                    Some(Command::Type { text })
                }
                "shell command" => {
                    let command = dialoguer::Input::new()
                        .with_prompt("enter the shell command to execute")
                        .interact_text()?;
                    Some(Command::Shell { command })
                }
                "sleep" => {
                    let secs = loop {
                        let Ok(secs) = dialoguer::Input::<f64>::new()
                        .with_prompt("enter sleep amount in seconds")
                        .interact_text() else { continue };
                        if secs.is_finite() && secs.is_sign_positive() {
                            break secs;
                        }
                    };

                    Some(Command::Sleep {
                        duration: std::time::Duration::from_secs_f64(secs),
                    })
                }
                "exit run" => break,
                _ => continue,
            };

            if let Some(command) = command {
                commands.push(command);
            }
        }

        Ok(Self { commands })
    }

    /// Executes the given command chain.
    pub(crate) fn execute(&self) -> anyhow::Result<()> {
        for command in &self.commands {
            command.execute()?;
        }

        Ok(())
    }

    /// Converts the command chain to a PDF file.
    pub(crate) fn to_pdf(&self, out_name: impl AsRef<Path>) -> anyhow::Result<()> {
        let key_codes = crate::key_codes::KeyCodes::new()?;

        let tempdir = tempfile::tempdir()?;
        let img_path = tempdir.path();

        let mut content = String::new();

        let mut img_idx = 0;

        for command in &self.commands {
            match command {
                Command::WaitForImage {
                    location,
                    image,
                    click,
                } => {
                    let mut path = img_path.to_path_buf();
                    path.push(format!("{img_idx}.png"));
                    image.save(&path)?;

                    content.push_str(&format!(
                        "== wait for{} image at {location}\n#image(\"{img_idx}.png\")\n\n",
                        if *click { " and click on" } else { "" },
                    ));

                    img_idx += 1;
                }
                Command::Sleep { duration } => {
                    content.push_str(&format!("== sleep for {duration:?}\n\n"));
                }
                Command::Shell { command } => {
                    content.push_str(&format!(
                        "== run shell command\n```bash\n{command}\n```\n\n"
                    ));
                }
                Command::PressKeys { keys } => {
                    content.push_str(&format!(
                        "== pressing keys\n{}\n\n",
                        keys.iter()
                            .map(|key| key_codes.reverse_lookup(*key).unwrap_or("<unknown key>"))
                            .collect::<Vec<_>>()
                            .join("\n")
                    ));
                }
                Command::Type { text } => {
                    content.push_str(&format!("== type text\n```text\n{text}\n```\n\n"));
                }
                Command::Click { position } => {
                    content.push_str(&format!("== click at {position}\n\n"));
                }
                Command::MouseMove { position } => {
                    content.push_str(&format!("== move mouse to {position}\n\n"));
                }
            }
        }

        let mut path = img_path.to_path_buf();
        path.push("joined.typ");

        std::fs::write(&path, content)?;

        std::process::Command::new("typst")
            .arg("compile")
            .arg(path)
            .arg(out_name.as_ref())
            .output()?;

        Ok(())
    }
}

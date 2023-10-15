//! Handles parsing and conversion of key codes.

use std::collections::BTreeMap;

/// A mapping from key names to their numbers.
#[derive(Debug)]
pub(crate) struct KeyCodes {
    /// The mapping from names to numbers.
    codes: BTreeMap<String, u32>,
    /// A sorted list of the available names.
    names_list: Vec<String>,
}

impl KeyCodes {
    /// Parses the available key codes.
    pub(crate) fn new() -> anyhow::Result<Self> {
        let mut codes = BTreeMap::new();

        use std::{
            fs::File,
            io::{BufRead as _, BufReader},
        };
        for line in BufReader::new(File::open("/usr/include/linux/input-event-codes.h")?).lines() {
            let line = line?;
            if let Some(rest_line) = line.strip_prefix("#define KEY_") {
                let mut parts = rest_line.split_whitespace();
                let Some(key_name) = parts.next() else { continue; };
                let Some(Ok(num)) = parts.next().map(|num_str| num_str.parse()) else { continue; };

                codes.insert(key_name.to_lowercase(), num);
            }
        }

        let names_list = codes.keys().cloned().collect();

        Ok(KeyCodes { codes, names_list })
    }

    /// Returns an iterator over the available key code names.
    pub(crate) fn codes(&self) -> &[String] {
        &self.names_list
    }

    /// Return the corresponding key code number for the given index into the names list.
    pub(crate) fn get_num(&self, index: usize) -> Option<u32> {
        self.codes.get(self.names_list.get(index)?).copied()
    }

    /// Lookup the name of a keycode.
    pub(crate) fn reverse_lookup(&self, keycode: u32) -> Option<&str> {
        for (name, &code) in &self.codes {
            if code == keycode {
                return Some(name);
            }
        }

        None
    }
}

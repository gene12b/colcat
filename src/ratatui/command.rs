use std::str::FromStr;

use strum::{EnumIter, IntoEnumIterator};

use crate::errors::ParseCommandError;

#[derive(Debug, EnumIter)]
pub enum Commands {
    Help,
    #[cfg(feature = "semantics")]
    Semantics(String),
}

impl Commands {
    pub fn help_text() -> String {
        let mut ret_string = String::new();
        for entry in Commands::iter() {
            let line = match entry {
                Self::Help => "help\t-\trepeat this help message.\n",
                #[cfg(feature = "semantics")]
                Self::Semantics(_) => {
                    "semantics [new_search_phrase]\t-\tchange the target phrase of interest to new_search_phrase and recompute.\n"
                }
            };
            ret_string.push_str(line);
        }
        ret_string
    }
}

impl FromStr for Commands {
    type Err = ParseCommandError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split_whitespace().filter(|val| !val.is_empty());
        let first_word = split.next().ok_or(ParseCommandError::EmptyCommand)?;
        let command = match first_word {
            "help" => Commands::Help,
            #[cfg(feature = "semantics")]
            "semantics" => {
                use itertools::Itertools;

                let new_sentence = split.join(" ");
                Commands::Semantics(new_sentence)
            }
            other => {
                return Err(ParseCommandError::UnsupportedCommand(other.to_string()));
            }
        };
        Ok(command)
    }
}

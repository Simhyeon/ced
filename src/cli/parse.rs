pub struct Parser {
    flags: Vec<Flag>,
    accept_flag_option: bool,
    start_index: usize,
    comma: bool,
    escaped : bool,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            flags : vec![],
            accept_flag_option: false,
            start_index : 0,
            comma : false,
            escaped : false,
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn parse_from_vec(&mut self, source: &Vec<impl AsRef<str>>) -> Vec<Flag> {
        for item in source {
            let should_break = self.find_word_variant(item.as_ref());
            if should_break {return std::mem::replace(&mut self.flags, vec![]);}
        }
        std::mem::replace(&mut self.flags, vec![])
    }

    pub fn parse_from_raw(&mut self, source : &str) -> Vec<Flag> {
        self.clear();
        let mut current_index = 0;
        for (idx,ch) in source.trim().chars().enumerate() {
            match ch {
                '\\' => self.escaped = true,
                '\'' => if self.escaped { 
                    self.escaped = false 
                } else {
                    self.comma = !self.comma
                },
                ' ' => {
                    if self.comma { continue; }
                    let should_break = self.find_word_variant(&source[self.start_index..idx]);
                    if should_break {
                        return std::mem::replace(&mut self.flags, vec![]);
                    }
                    self.start_index = idx;
                } , // End of word
                _ => ()
            }
            current_index = idx;
            // End of socket
        }

        // If start index is not updated ( "No word has been detected" )
        // and source's length is same with current index
        if current_index == source.len().max(1) - 1 {
            self.find_word_variant(&source[self.start_index..current_index + 1]);
        }

        std::mem::replace(&mut self.flags, vec![])
    }

    /// Check word variant
    ///
    /// * - Return : If loop should break
    fn find_word_variant(&mut self, word: &str) -> bool {
        // Add argument
        if !self.accept_flag_option && !word.starts_with("-") {
            self.flags.push(Flag::argument(word))
        } else { // Add other flag
            // You cannot set accept_flag_option without setting a flag 
            // thus it is safe to unwrap
            if self.accept_flag_option {
                self.flags.last_mut().unwrap().option = word.to_string();
                self.accept_flag_option = false; 
                return false;
            }

            let flag = Self::match_word(word);

            if flag.early_exit {
                self.flags = vec![flag];
                return true;
            } 

            if flag.need_option {
                self.accept_flag_option = true;
            }

            if flag.ftype != FlagType::None {
                self.flags.push(flag);
            }
        }

        return false;
    }

    fn match_word(word: &str) -> Flag {
        match word.trim() {
            "--version" | "-v" => Flag::version(),
            "--help" | "-h" => Flag::help(),
            "--command" | "-c" => Flag::command(),
            "--schema" | "-s" => Flag::schema(),
            "--confirm" | "-C" => Flag::confirm(),
            _ => Flag::empty(),
        }
    }
}

#[derive(Debug)]
pub struct Flag {
    pub ftype: FlagType,
    pub need_option: bool,
    pub option: String,
    pub early_exit: bool,
}

impl Flag {
    pub fn empty() -> Self {
        Self {
            ftype : FlagType::None,
            need_option : false,
            option: String::new(),
            early_exit: false,
        }
    }

    pub fn argument(arg: &str) -> Self {
        Self {
            ftype : FlagType::Argument,
            need_option : true,
            option: arg.to_string(),
            early_exit: false,
        }
    }

    pub fn schema() -> Self {
        Self {
            ftype : FlagType::Schema,
            need_option : true,
            option: String::new(),
            early_exit: false,
        }
    }

    pub fn command() -> Self {
        Self {
            ftype : FlagType::Command,
            need_option : true,
            option: String::new(),
            early_exit: false,
        }
    }

    pub fn confirm() -> Self {
        Self {
            ftype : FlagType::Confirm,
            need_option : false,
            option: String::new(),
            early_exit: false,
        }
    }

    pub fn version() -> Self {
        Self {
            ftype: FlagType::Version,
            need_option : false,
            option: String::new(),
            early_exit: true,
        }
    }

    pub fn help() -> Self {
        Self {
            ftype: FlagType::Help,
            need_option : false,
            option: String::new(),
            early_exit: true,
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum FlagType {
    Argument,
    Command,
    Confirm,
    Help,
    Schema,
    Version,
    None,
}

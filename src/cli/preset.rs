use crate::{CedError, CedResult};
use dcsv::{ValueLimiter, LIMITER_ATTRIBUTE_LEN};
use std::{collections::HashMap, path::PathBuf};

const PRESET_FILE_NAME: &str = ".ced_preset.csv";

pub struct Preset {
    list: HashMap<String, ValueLimiter>,
}

impl Preset {
    pub fn empty() -> Self {
        Self {
            list: HashMap::new(),
        }
    }
    pub fn new(use_defualt: bool) -> CedResult<Self> {
        let mut instance = Self {
            list: HashMap::new(),
        };
        if use_defualt {
            instance.append_default()?;
        }

        // Extend from file
        instance.extend_from_file()?;
        Ok(instance)
    }

    pub fn get(&self, preset: &str) -> Option<&ValueLimiter> {
        self.list.get(preset)
    }

    pub fn extend_from_file(&mut self) -> CedResult<()> {
        let preset_path = get_global_preset_file();
        if !preset_path.exists() {
            return Ok(());
        }
        let preset_src = std::fs::read_to_string(preset_path)
            .map_err(|err| CedError::io_error(err, "Faeild to read preset path"))?;
        let presets = Self::parse_preset(&preset_src)?;
        self.list.extend(IntoIterator::into_iter(presets));
        Ok(())
    }

    // TODO
    // This should be different behaviour because regex pattern can be alchaic
    fn parse_preset(source: &str) -> CedResult<Vec<(String, ValueLimiter)>> {
        let mut limiters = vec![];
        for line in source.lines() {
            let csv = dcsv::utils::csv_row_to_vector(line, None, false);
            if csv.len() != LIMITER_ATTRIBUTE_LEN + 1 {
                return Err(CedError::InvalidRowData(format!(
                    "Given line \"{}\" doesn't include necessary limiter data",
                    line
                )));
            }
            let preset_name = csv[0].to_owned();
            let limiter = ValueLimiter::from_line(&csv[1..].to_vec())?;
            limiters.push((preset_name, limiter));
        }
        Ok(limiters)
    }

    fn append_default(&mut self) -> CedResult<()> {
        let default = IntoIterator::into_iter([
            (
                "text".to_owned(),
                ValueLimiter::from_line(&["text", "", "", ""])?,
            ),
            (
                "number".to_owned(),
                ValueLimiter::from_line(&["number", "", "", ""])?,
            ),
            (
                "float".to_owned(),
                ValueLimiter::from_line(&["text", "0.0", "", r#"[+-]?([0-9]*[.])?[0-9]+"#])?,
            ),
            (
                "email".to_owned(),
                ValueLimiter::from_line(&[
                    "text",
                    "johndoe@mail.com",
                    "",
                    r#"^([a-zA-Z0-9._%-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,6})*$"#,
                ])?,
            ),
            (
                "date".to_owned(),
                ValueLimiter::from_line(&[
                    "text",
                    "2000-01-01",
                    "",
                    r#"([12]\d{3}-(0[1-9]|1[0-2])-(0[1-9]|[12]\d|3[01]))"#,
                ])?,
            ),
            (
                "time".to_owned(),
                ValueLimiter::from_line(&[
                    "text",
                    "00:00:00",
                    "",
                    r#"^(?:(?:([01]?\d|2[0-3]):)?([0-5]?\d):)?([0-5]?\d)$"#,
                ])?,
            ),
            (
                "url".to_owned(),
                ValueLimiter::from_line(&[
                    "text",
                    "http://john.doe",
                    "",
                    r#"[(http(s)?)://(www\.)?a-zA-Z0-9@:%._\+~#=]{2,256}\.[a-z]{2,6}\b([-a-zA-Z0-9@:%_\+.~#?&//=]*)"#,
                ])?,
            ),
        ]);
        self.list.extend(default);
        Ok(())
    }
}

fn get_global_preset_file() -> PathBuf {
    #[cfg(not(target_os = "windows"))]
    let preset_path: String = std::env::var("HOME").expect("Failed to retrieve home directory");
    #[cfg(target_os = "windows")]
    let preset_path: String =
        std::env::var("APPDATA").expect("Failed to retrieve app data directory");
    PathBuf::from(preset_path).join(PRESET_FILE_NAME)
}

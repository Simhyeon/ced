use crate::utils;
use crate::CommandType;

const BINARY_HELP: &str = include_str!("../help/bin.txt");
const HELP_TEXT: &str = include_str!("../help/all.txt");

pub fn print_binary_help_text() {
    utils::write_to_stdout(BINARY_HELP).expect("Failed to print to terminal");
}

pub fn print_help_text() {
    utils::write_to_stdout(HELP_TEXT).expect("Failed to print to terminal");
}

pub fn print_version() {
    print!(include_str!("../help/version.txt"));
}

pub fn print_command_help(command: CommandType) {
    let out = match command {
        CommandType::Version => include_str!("../help/raw/01_version"),
        CommandType::Help => include_str!("../help/raw/02_help"),
        CommandType::Undo => include_str!("../help/raw/22_undo"),
        CommandType::Redo => include_str!("../help/raw/23_redo"),
        CommandType::Create => include_str!("../help/raw/07_create"),
        CommandType::Write => include_str!("../help/raw/04_write"),
        CommandType::Import => include_str!("../help/raw/03_import"),
        CommandType::ImportRaw => include_str!("../help/raw/03_import_raw"),
        CommandType::Export => include_str!("../help/raw/04_export"),
        CommandType::AddRow => include_str!("../help/raw/08_add_row"),
        CommandType::AddColumn => include_str!("../help/raw/09_add_column"),
        CommandType::DeleteRow => include_str!("../help/raw/13_delete_row"),
        CommandType::DeleteColumn => include_str!("../help/raw/14_delete_column"),
        CommandType::EditCell => include_str!("../help/raw/10_edit"),
        CommandType::EditColumn => include_str!("../help/raw/12_edit_column"),
        CommandType::RenameColumn => include_str!("../help/raw/15_rename_column"),
        CommandType::EditRow => include_str!("../help/raw/11_edit_row"),
        CommandType::EditRowMultiple => include_str!("../help/raw/11_edit_row"),
        CommandType::MoveRow => include_str!("../help/raw/16_move"),
        CommandType::MoveColumn => include_str!("../help/raw/17_move_column"),
        CommandType::Exit => include_str!("../help/raw/32_quit"),
        // TODO
        // Unimplemented!
        CommandType::Execute => include_str!("../help/raw/32_quit"),
        CommandType::Print => include_str!("../help/raw/05_print"),
        CommandType::PrintCell => include_str!("../help/raw/05_print_cell"),
        CommandType::PrintRow => include_str!("../help/raw/05_print_row"),
        CommandType::PrintColumn => include_str!("../help/raw/06_print_column"),
        CommandType::Limit => include_str!("../help/raw/18_limit"),
        // TODO
        // Unimplemented!
        CommandType::LimitPreset => include_str!("../help/raw/18_limit"),
        CommandType::Schema => include_str!("../help/raw/19_schema"),
        CommandType::SchemaInit => include_str!("../help/raw/21_schema_init"),
        CommandType::SchemaExport => include_str!("../help/raw/20_schema_export"),
        CommandType::History => include_str!("../help/raw/24_history"),
        CommandType::None => "No such command to print a help message.\n",
    };
    utils::write_to_stdout(out).expect("Failed to write to terminal");
}

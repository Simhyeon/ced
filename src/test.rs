// TODO
// It is "not" trivial...

use crate::CedResult;
use crate::Command;
use crate::Processor;

const TEST1_CSV: &str = include_str!("../test/test1.csv");

#[test]
fn command_test() -> CedResult<()> {
    let mut processor = Processor::new();

    // TEST1.csv
    processor.import_from_string(TEST1_CSV, true)?;
    processor.execute_command(&Command::from_str("print")?)?;
    Ok(())
}

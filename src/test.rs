// TODO
// It is "not" trivial...

use crate::CedResult;
//use crate::Processor;

//const TEST1_CSV: &str = include_str!("../test/test1.csv");

#[test]
fn command_test() -> CedResult<()> {
    use crate::Processor;
    let mut processor = Processor::new();
    processor
        .import_from_file("test.csv", true, None, false)
        .unwrap();
    let page_name = processor.get_cursor().unwrap();
    processor
        .add_row_from_strings(
            &page_name,
            processor.last_row_index(&page_name)?,
            &["a", "b"],
        )
        .unwrap();

    processor.overwrite_to_file(&page_name, true).unwrap();
    Ok(())
}

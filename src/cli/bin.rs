use ced::CedResult;

pub fn main() -> CedResult<()> {
    #[cfg(feature = "cli")]
    ced::start_main_loop()?;
    Ok(())
}

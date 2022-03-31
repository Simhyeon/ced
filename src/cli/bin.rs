#[cfg(feature="cli")]
use ced::error::CedResult;

#[cfg(feature="cli")]
pub fn main() -> CedResult<()> {
    let args: Vec<String> = std::env::args().collect();
    if let Some(first_arg) = args.get(1) {
        match first_arg.as_str() {
            "--version" | "-v" => {
                println!("ced, 0.1.0");
                return Ok(());
            },
            "--help" | "-h" => {
                println!("{}", include_str!("../help.txt"));
                return Ok(());
            }
            _ => ()
        }
    }

    use ced::cli::CommandLoop;
    let mut command_loop = CommandLoop::new();
    command_loop.start_loop().err().map(|err| println!("{}",err));
    Ok(())
}

#[cfg(not(feature="cli"))]
pub fn main() { }

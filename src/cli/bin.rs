#[cfg(feature="cli")]
pub fn main() {
    use ced::cli::CommandLoop;
    let mut command_loop = CommandLoop::new();
    command_loop.start_loop().err().map(|err| println!("{}",err));
}

#[cfg(not(feature="cli"))]
pub fn main() { }

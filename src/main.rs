use renameit_gui::{GuiError, run};

fn main() -> Result<(), GuiError> {
    run(std::env::args().nth(1))
}

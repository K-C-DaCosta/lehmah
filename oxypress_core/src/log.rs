#[derive(Copy, Clone)]
pub enum ConsoleColors {
    BLACK = 30,
    RED = 31,
    GREEN = 32,
    YELLOW = 33,
    BLUE = 34,
    MAGENTA = 35,
    CYAN = 36,
    WHITE = 37,
    DEFAULT = 39, 
}

#[macro_export]
macro_rules! loggy {
    ( $head:expr, $($rest:tt)+ ) => {
        let log_timestamp = chrono::Utc::now().format("%d/%m/%Y_%H:%M:%S");
        std::print!(
            "[{}] \x1b[{}m",
            log_timestamp,
            $head as ConsoleColors as isize
        );
        // TO DO: This could be more efficient, but its good for now
        std::print!("{}",&format_args!($($rest)*).to_string());
        std::print!("\x1b[0m\n");
    };
}

#[test]
fn sanity_test() {
    loggy!(ConsoleColors::YELLOW, "Hello world, im Yellow!");
    loggy!(ConsoleColors::GREEN, "Hello world, im Green!");
    loggy!(ConsoleColors::RED, "Hello world, im Red!");
    loggy!(ConsoleColors::BLUE, "Hello world, im Blue!");
    loggy!(ConsoleColors::DEFAULT, "Hello world, im Default!");
}

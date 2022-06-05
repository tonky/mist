use ansi_term::Colour;

pub fn info(str: &str) -> () {
    println!("{}", str);
}

pub fn warning(str: &str) -> () {
    println!(
        "{} {}",
        Colour::Yellow.paint("Warn:"),
        str
    );
}

pub fn error(str: &str) -> () {
    println!(
        "{} {}",
        Colour::Red.paint("Err:"),
        str
    );
}
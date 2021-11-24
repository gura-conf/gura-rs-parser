// Basic Gura parser usage example
use gura::{errors::Error, parse};

fn main() {
    let gura_string = r##"
# This is a Gura document.
title: "Gura Example"

some_invalid: $non_existent_var 
"##;

    // Parse: transforms a Gura string into a dictionary
    match parse(&gura_string) {
        Ok(parsed) => {
            println!("Title -> {}", parsed["title"]);
        }
        Err(e) => {
            println!("Error: {}", e); // Error implements fmt::Display

            match e.kind {
                Error::ParseError => println!("Syntax is wrong!"),
                Error::VariableNotDefinedError => println!("A non defined variable was used! "),
                Error::InvalidIndentationError => println!("Indentation is invalid!"),
                Error::DuplicatedVariableError => {
                    println!("A variable was defined more than once!")
                }
                Error::DuplicatedKeyError => println!("A key was defined more than once!"),
                Error::FileNotFoundError => println!("An imported file does not exist!"),
                Error::DuplicatedImportError => {
                    println!("The same Gura file was imported more than once!")
                }
            }
        }
    }
}

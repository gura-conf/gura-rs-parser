// Basic Gura parser usage example
use gura::{dump, parse, GuraType};

fn main() {
    let gura_string = "
# This is a Gura document.
title: \"Gura Example\"

an_object:
    username: \"Stephen\"
    pass: \"Hawking\"

# Line breaks are OK when inside arrays
hosts: [
  \"alpha\",
  \"omega\"
]"
    .to_string();

    // Parse: transforms a Gura string into a dictionary
    let parsed = parse(&gura_string).unwrap();

    // Debug and Display
    // println!("{:#?}", parsed);
    // println!("{}", parsed);

    // Access a specific field
    println!("Title -> {}", parsed["title"]);

    // Iterate over structure
    println!("\nHosts:");
    if let GuraType::Array(hosts) = &parsed["hosts"] {
        for host in hosts.iter() {
            println!("Host -> {}", *host);
        }
    }

    // Dump: transforms a dictionary into a Gura string
    let string_again = dump(&parsed);
    println!("\n+++++ Dump result +++++");
    println!("{}", string_again);
}

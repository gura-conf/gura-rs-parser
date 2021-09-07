// Basic Gura parser usage example
use gura::{dump, parse, GuraType};

fn main() {
    let gura_string = r##"
# This is a Gura document.
title: "Gura Example"

an_object:
    username: "Stephen"
    pass: "Hawking"

# Line breaks are OK when inside arrays
hosts: [
  "alpha",
  "omega"
]"##;

    // Parse: transforms a Gura string into a dictionary
    let parsed = parse(&gura_string).unwrap();

    // Debug and Display
    // println!("{:#?}", parsed);
    // println!("{}", parsed);

    // Access a specific field
    println!("Title -> {}", parsed["title"]);

    // You can check if object contains a key
    if parsed.contains_key("an_object") {
        println!("\nGura object contains 'an_object' key!");
    }

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

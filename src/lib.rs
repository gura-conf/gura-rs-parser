//! ![](https://github.com/gura-conf/gura/blob/master/logos/gura-200.png?raw=true)
//!
//! # Gura Rust
//!
//! [Gura](https://gura.netlify.app/) is a file format for configuration files. Gura is as readable as YAML and simple as TOML. Its syntax is clear and powerful, yet familiar for YAML/TOML users.
//!
//! This crate parses Gura strings into Rust structures and vice versa:
//!
//! ```
//! use gura::{dump, parse, GuraType};
//!
//! let gura_string = r##"
//! title: "Gura Example"
//!
//! an_object:
//!     username: "Stephen"
//!     pass: "Hawking"
//!
//! hosts: [
//!   "alpha",
//!   "omega"
//! ]"##;
//!
//! // Parse: transforms a Gura string into a dictionary
//! let parsed = parse(&gura_string).unwrap();
//!
//! // Debug and Display
//! // println!("{:#?}", parsed);
//! // println!("{}", parsed);
//!
//! // Access a specific field
//! println!("Title -> {}", parsed["title"]);
//!
//! // Iterate over structure
//! println!("\nHosts:");
//! if let GuraType::Array(hosts) = &parsed["hosts"] {
//!     for host in hosts.iter() {
//!         println!("Host -> {}", *host);
//!     }
//! }
//!
//! // Dump: transforms a dictionary into a Gura string
//! let string_again = dump(&parsed);
//! println!("\n+++++ Dump result +++++");
//! println!("{}", string_again);
//! ```
//!
//! ## Easy creation, easy access
//!
//! Using macros and indexing, it's easy to work with the data.
//!
//! ```
//! use gura::{object, dump, GuraType};
//!
//! let object = object! {
//!     a_number: 55,
//!     nested: {
//!         array: [1, 2, 3],
//!         nested_ar: [1, [2, 3], 4]
//!     },
//!     a_string: "Gura Rust"
//! };
//!
//! // Access a specific field
//! assert_eq!(object["a_number"], 55);
//! assert_eq!(object["a_string"], "Gura Rust");
//!
//! // Iterate over structure
//! println!("\nNested/Array:");
//! if let GuraType::Array(numbers) = &object["nested"]["array"] {
//!     for number in numbers.iter() {
//!         println!("Number in array -> {}", *number);
//!     }
//! }
//!
//! // Dump: transforms a dictionary into a Gura string
//! let object_string = dump(&object);
//! println!("\n+++++ Dump result +++++");
//! println!("{}", object_string);
//! ```

pub mod errors;
pub mod macros;
pub mod parser;
mod pretty_print_float;

// Re-exporting
pub use self::parser::dump;
pub use self::parser::parse;
pub use self::parser::GuraType;

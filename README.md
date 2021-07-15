# Gura Rust parser

This repository contains the implementation of a [Gura][gura] (compliant with version 1.0.0) format parser for Rust lang.

**[Documentation](https://docs.rs/gura/) -**
**[Cargo](https://crates.io/crates/gura) -**


## Installation

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
gura = "0.1.0"
```


## Usage

```rust
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
```


## Contributing

All kind of contribution is welcome! If you want to contribute just:

1. Fork this repository.
1. Create a new branch and introduce there your new changes.
1. Make a Pull Request!


### Tests

To run all the tests: `cargo test`


## Licence

This repository is distributed under the terms of the MIT license.


[gura]: https://github.com/gura-conf/gura
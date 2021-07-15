// Basic Gura object creation example
use gura::{dump, object, GuraType};

fn main() {
    let object = object! {
        a_number: 55,
        nested: {
            array: [1, 2, 3],
            nested_ar: [1, [2, 3], 4]
        },
        a_string: "Gura Rust"
    };

    // Access a specific field
    println!("Number -> {}", object["a_number"]);
    println!("String -> {}", object["a_string"]);

    // Iterate over structure
    println!("\nNested/Array:");
    if let GuraType::Array(numbers) = &object["nested"]["array"] {
        for number in numbers.iter() {
            println!("Number in array -> {}", *number);
        }
    }

    // Dump: transforms a dictionary into a Gura string
    let object_string = dump(&object);
    println!("\n+++++ Dump result +++++");
    println!("{}", object_string);
}

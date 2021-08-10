use gura::{parse, GuraType};

#[derive(Debug)]
struct TangoSinger {
    name: String,
    surname: String,
    year_of_birth: u16,
}

fn main() {
    // Until Serde-Gura implementation is finished you can make manual struct instantiation
    let gura_string = r##"
# This is a Gura document.

# Array of objects
tango_singers: [
    user1:
        name: "Carlos"
        surname: "Gardel"
        year_of_birth: 1890,
    user2:
        name: "Aníbal"
        surname: "Troilo"
        year_of_birth: 1914
]"##;

    // Parse: transforms a Gura string into a dictionary
    let parsed = parse(&gura_string).unwrap();

	// Lets make an array of singers
    if let GuraType::Array(tango_singers) = &parsed["tango_singers"] {
        let mut tango_singers_structs: Vec<TangoSinger> =
            Vec::with_capacity(tango_singers.capacity());

        // Iterate over structure
        for tango_singer in tango_singers {
            // Discards object key
            if let GuraType::Object(key_values) = tango_singer {
                let (_singer_key, singer_props) = key_values.iter().next().unwrap();

                // Inside the for loop
                let year_of_birth: u16 = match singer_props["year_of_birth"] {
                    GuraType::Integer(value) => value as u16,
                    GuraType::BigInteger(value) => value as u16,
                    _ => panic!("Gura text is not a valid array of tango singers!"),
                };

                let my_struct = TangoSinger {
                    name: singer_props["name"].to_string(),
                    surname: singer_props["surname"].to_string(),
                    year_of_birth,
                };

                tango_singers_structs.push(my_struct);
            } else {
                panic!("Gura text is not a valid array of tango singers!")
            }
        }

        println!("Tango singers:");
        println!("{:#?}", tango_singers_structs);
    }
}

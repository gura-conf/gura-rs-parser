use gura::{dump, parse};

fn main() {
    let str = "foo: [
    bar:
        baz: [
            far: [
                faz: \"foo\"
            ],
            far: \"faz\",
            far: \"faz\"
        ],
    [empty, empty, empty],
    [
        foo:
            hi: \"bar\"
            jeje: [
                foo: [
                    bar:
                        baz: [
                            far: [
                                faz: \"foo\"
                            ],
                            far: \"faz\",
                            far: \"faz\"
                        ],
                    [empty, empty, empty],
                    [
                        foo:
                            hi: \"bar\"
                            jeje: []
                    ]
                ]
            ]
    ]
]"
    .to_string();

    let parsed = parse(&str).unwrap();
    let dumped = dump(&parsed);
    assert_eq!(str.trim(), dumped.trim());
}

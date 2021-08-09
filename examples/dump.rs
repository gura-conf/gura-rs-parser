use gura::{dump, parse};

fn main() {
    let str = r##"foo: [
    bar:
        baz: [
            far: [
                faz: "foo"
            ],
            far: "faz",
            far: "faz"
        ],
    [empty, empty, empty],
    [
        foo:
            hi: "bar"
            bye: [
                foo: [
                    bar:
                        baz: [
                            far: [
                                faz: "foo"
                            ],
                            far: "faz",
                            far: "faz"
                        ],
                    [empty, empty, empty],
                    [
                        foo:
                            hi: "bar"
                            bye: []
                    ]
                ]
            ]
    ]
]"##;

    let parsed = parse(&str).unwrap();
    let dumped = dump(&parsed);
    assert_eq!(str.trim(), dumped.trim());
}

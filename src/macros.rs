use crate::parser::GuraType;

/// Helper to cast values to Gura types
pub trait Attribute {
    fn process(&self) -> GuraType;
}

impl Attribute for bool {
    fn process(&self) -> GuraType { GuraType::Bool(*self) }
}

impl Attribute for f32 {
    fn process(&self) -> GuraType { GuraType::Float(*self as f64) }
}

impl Attribute for f64 {
    fn process(&self) -> GuraType { GuraType::Float(*self) }
}

impl Attribute for isize {
    fn process(&self) -> GuraType { GuraType::Integer(*self) }
}

impl Attribute for &str {
    fn process(&self) -> GuraType { GuraType::String(self.to_string()) }
}

impl Attribute for String {
    fn process(&self) -> GuraType { GuraType::String(self.clone()) }
}

/// Helper macro for creating instances of `GuraType::Array`.
// TODO: add example and make private
#[macro_export]
macro_rules! array {
    [] => ($crate::GuraType::Array(Vec::new()));

    // Handles for token tree items
    [@ITEM($( $i:expr, )*) $item:tt, $( $cont:tt )+] => {
        $crate::array!(
            @ITEM($( $i, )* $crate::value!($item), )
            $( $cont )*
        )
    };
    (@ITEM($( $i:expr, )*) $item:tt,) => ({
        $crate::array!(@END $( $i, )* $crate::value!($item), )
    });
    (@ITEM($( $i:expr, )*) $item:tt) => ({
        $crate::array!(@END $( $i, )* $crate::value!($item), )
    });

    // Handles for expression items
    [@ITEM($( $i:expr, )*) $item:expr, $( $cont:tt )+] => {
        $crate::array!(
            @ITEM($( $i, )* $crate::value!($item), )
            $( $cont )*
        )
    };
    (@ITEM($( $i:expr, )*) $item:expr,) => ({
        $crate::array!(@END $( $i, )* $crate::value!($item), )
    });
    (@ITEM($( $i:expr, )*) $item:expr) => ({
        $crate::array!(@END $( $i, )* $crate::value!($item), )
    });

    // Construct the actual array
    (@END $( $i:expr, )*) => ({
        let size = 0 $( + {let _ = &$i; 1} )*;
        let mut array = Vec::with_capacity(size);

        $(
            array.push($i.into());
        )*

        $crate::parser::GuraType::Array(array)
    });

    // Entry point to the macro
    ($( $cont:tt )+) => {
        $crate::array!(@ITEM() $($cont)*)
    };
}

#[macro_export]
/// Helper crate for converting types into `GuraType`. It's used
/// internally by the `object!` and `array!` macros.
// TODO: make private
macro_rules! value {
    ( null ) => { $crate::parser::GuraType::Null };
    ( [$( $token:tt )*] ) => {
        // 10
        $crate::array![ $( $token )* ]
    };
    ( {$( $token:tt )*} ) => {
        $crate::object!{ $( $token )* }
    };
    { $value:expr } => { $crate::macros::Attribute::process(&$value) };
}

/// Helper macro for creating instances of `GuraType::Object`.
// TODO: add example
#[macro_export]
macro_rules! object {
    // Empty object.
    {} => ($crate::parser::GuraType::Object(HashMap::new()));

    // Handles for different types of keys
    (@ENTRY($( $k:expr => $v:expr, )*) $key:ident: $( $cont:tt )*) => {
        $crate::object!(@ENTRY($( $k => $v, )*) stringify!($key).to_string() => $($cont)*)
    };
    (@ENTRY($( $k:expr => $v:expr, )*) $key:literal: $( $cont:tt )*) => {
        $crate::object!(@ENTRY($( $k => $v, )*) $key => $($cont)*)
    };
    (@ENTRY($( $k:expr => $v:expr, )*) [$key:expr]: $( $cont:tt )*) => {
        $crate::object!(@ENTRY($( $k => $v, )*) $key => $($cont)*)
    };

    // Handles for token tree values
    (@ENTRY($( $k:expr => $v:expr, )*) $key:expr => $value:tt, $( $cont:tt )+) => {
        $crate::object!(
            @ENTRY($( $k => $v, )* $key => $crate::value!($value), )
            $( $cont )*
        )
    };
    (@ENTRY($( $k:expr => $v:expr, )*) $key:expr => $value:tt,) => ({
        $crate::object!(@END $( $k => $v, )* $key => $crate::value!($value), )
    });
    (@ENTRY($( $k:expr => $v:expr, )*) $key:expr => $value:tt) => ({
        $crate::object!(@END $( $k => $v, )* $key => $crate::value!($value), )
    });

    // Handles for expression values
    (@ENTRY($( $k:expr => $v:expr, )*) $key:expr => $value:expr, $( $cont:tt )+) => {
        $crate::object!(
            @ENTRY($( $k => $v, )* $key => $crate::value!($value), )
            $( $cont )*
        )
    };
    (@ENTRY($( $k:expr => $v:expr, )*) $key:expr => $value:expr,) => ({
        $crate::object!(@END $( $k => $v, )* $key => $crate::value!($value), )
    });

    (@ENTRY($( $k:expr => $v:expr, )*) $key:expr => $value:expr) => ({
        $crate::object!(@END $( $k => $v, )* $key => $crate::value!($value), )
    });

    // Construct the actual object
    (@END $( $k:expr => $v:expr, )*) => ({
        let size = 0 $( + {let _ = &$k; 1} )*;
        // let mut object = $crate::object::Object::with_capacity(size);
        let mut object: std::collections::HashMap<std::string::String, Box<GuraType>> = std::collections::HashMap::new();

        $(
            object.insert($k, Box::new($v));
        )*

        $crate::parser::GuraType::Object(object)
    });

    // Entry point to the macro
    ($key:tt: $( $cont:tt )+) => {
        $crate::object!(@ENTRY() $key: $($cont)*)
    };

    // Legacy macro
    ($( $k:expr => $v:expr, )*) => {
        $crate::object!(@END $( $k => $crate::value!($v), )*)
    };
    ($( $k:expr => $v:expr ),*) => {
        $crate::object!(@END $( $k => $crate::value!($v), )*)
    };
}
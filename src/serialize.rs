//! Thismodule provides functionality for serializing data structures
//! to JSON format and implementing `Display` for them.

/// This trait defines a method for converting a struct to a JSON string.
pub trait Json {
    fn to_json(&self) -> String;
}

#[macro_export]
macro_rules! impl_json {
    ($ty:ident { $($field:ident : $key:expr),* $(,)? }) => {
        impl $crate::serialize::Json for $ty {
            #[allow(unused_assignments)]
            fn to_json(&self) -> String {
                let mut s = String::new();
                let mut first = true;

                s.push('{');
                $(
                    if !first { s.push_str(", "); }
                    first = false;
                    s.push('"');
                    s.push_str($key);
                    s.push_str("\": \"");
                    s.push_str(&self.$field);
                    s.push('"');
                )*
                s.push('}');
                s
            }
        }

        impl Display for $ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.to_json())
            }
        }
    };
}

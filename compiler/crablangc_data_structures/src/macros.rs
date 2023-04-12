#[macro_export]
macro_rules! enum_from_u32 {
    ($(#[$attr:meta])* pub enum $name:ident {
        $($(#[$var_attr:meta])* $variant:ident = $e:expr,)*
    }) => {
        $(#[$attr])*
        pub enum $name {
            $($(#[$var_attr])* $variant = $e),*
        }

        impl $name {
            pub fn from_u32(u: u32) -> Option<$name> {
                $(if u == $name::$variant as u32 {
                    return Some($name::$variant)
                })*
                None
            }
        }
    };
    ($(#[$attr:meta])* pub enum $name:ident {
        $($(#[$var_attr:meta])* $variant:ident,)*
    }) => {
        $(#[$attr])*
        pub enum $name {
            $($(#[$var_attr])* $variant,)*
        }

        impl $name {
            pub fn from_u32(u: u32) -> Option<$name> {
                $(if u == $name::$variant as u32 {
                    return Some($name::$variant)
                })*
                None
            }
        }
    }
}

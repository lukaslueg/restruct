#[macro_export]
macro_rules! strct {
    ($name:ident $($fmt:literal)* $(#$meta:meta)*) => {
        #[derive(restruct_derive::Struct)]
        $(
            #[fmt = $fmt]
        )*
        $(
            #[$meta]
        )*
        struct $name;
    };
    ($($fmt:literal)*) => {
        strct!(Foo $($fmt)*);
    };
}

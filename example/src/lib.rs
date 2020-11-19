#![feature(const_fn)]
#![feature(const_fn_transmute)]

extern "C" {
    fn reach_turtle() -> *const u8;
    fn reach_lowest_turtle() -> *const u8;
}

#[derive(restruct_derive::Struct)]
#[fmt = ">Ih?"]
pub struct Example;

#[allow(clippy::declare_interior_mutable_const)]
const _EXAMPLE: <Example as restruct::Struct>::Unpacked =
    Example::unpack(*include_bytes!("example.bin"));

pub mod world {
    #[derive(restruct_derive::Struct)]
    #[fmt = "@bhlbibqBHLbIbQ3s"]
    pub struct Turtle;

    impl Turtle {
        pub fn retrieve() -> <Self as restruct::Struct>::Unpacked {
            unsafe { Self::from_raw(super::reach_turtle()) }
        }
    }

    #[derive(restruct_derive::Struct)]
    #[fmt = "bQh"]
    pub struct LowerTurtle;

    #[derive(restruct_derive::Struct)]
    #[fmt = "i2`LowerTurtle`"]
    pub struct LowestTurtle;

    impl LowestTurtle {
        pub fn retrieve() -> <Self as restruct::Struct>::Unpacked {
            unsafe { Self::from_raw(super::reach_lowest_turtle()) }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn const_init() {
        assert_eq!(super::_EXAMPLE, (0xdead_c0de, 400, true));
    }

    #[test]
    fn reach_bottom_most_turtle() {
        let u = super::world::LowestTurtle::retrieve();
        assert_eq!(u, (-1, (100, 127, 128), (100, 10_000_000_000, -32000)));
    }

    #[test]
    fn reach_the_turtle() {
        let u = super::world::Turtle::retrieve();
        assert_eq!(u.0, 100);
        assert_eq!(u.1, -32000);
        assert_eq!(u.2, -200_000_000);
        assert_eq!(u.3, 127);
        assert_eq!(u.4, -1_000_000_000);
        assert_eq!(u.5, 100);
        assert_eq!(u.6, 10_000_000_000);
        assert_eq!(u.7, 128);
        assert_eq!(u.8, 32000);
        assert_eq!(u.9, 400_000_000);
        assert_eq!(u.10, 3);
        assert_eq!(u.11, 300_000_000);
        assert_eq!(u.12, 4);
        assert_eq!(u.13, 100_000_000_000);
        assert_eq!(u.14, [1, 2, 3]);
    }
}

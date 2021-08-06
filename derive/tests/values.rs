#[macro_use]
mod common;

#[test]
fn simple() {
    strct!(">hhl");
    let inp = (1, 2, 3);
    let expected = [0, 1, 0, 2, 0, 0, 0, 3];
    let packed = Foo::pack(inp);
    assert_eq!(packed, expected);
    let unpacked = Foo::unpack(packed);
    assert_eq!(unpacked, inp);
}

macro_rules! known_values {
    ($modifier:literal $fmt:literal $value:expr, $fix:ident) => {
        {
            strct!($modifier $fmt);
            let buf = Foo::pack(($value, ));
            assert_eq!(buf, $fix);
            let outp = Foo::unpack(buf);
            assert_eq!(outp, ($value, ));
        }
    };
    ($name:ident $fmt:literal, $(($value:expr, $le_fix:expr)),+) => {
        #[test]
        fn $name() {
        $(
            let le = $le_fix;
            let mut be = $le_fix;
            be.reverse();
            known_values!("<" $fmt $value, le);
            known_values!(">" $fmt $value, be);
            known_values!("!" $fmt $value, be);
            #[cfg(target_endian="little")] known_values!("=" $fmt $value, le);
            #[cfg(target_endian="big")] known_values!("=" $fmt $value, be);
        )+
        }
    }
}

mod known_values {
    known_values!(chr "b", (7, [7]), (-7, [249]));
    known_values!(uchar "B", (7, [7]), (249, [249]));
    known_values!(short "h", (700, [188, 2]), (-700, [68, 253]));
    known_values!(ushort "H" ,(700, [188, 2]), (64836, [68, 253]));
    known_values!(int "i", (70_000_000, [128, 29, 44, 4]), (-70_000_000, [128, 226, 211, 251]));
    known_values!(uint "I", (70_000_000, [128, 29, 44, 4]), (4_224_967_296, [128, 226, 211, 251]));
    known_values!(long "l", (70_000_000, [128, 29, 44, 4]), (-70_000_000, [128, 226, 211, 251]));
    known_values!(ulong "L", (70_000_000, [128, 29, 44, 4]), (4_224_967_296, [128, 226, 211, 251]));
    known_values!(float "f", (2.0, [0, 0, 0, 64]), (-2.0, [0, 0, 0, 192]));
    known_values!(double "d", (2.0, [0, 0, 0, 0, 0, 0, 0, 64]), (-2.0, [0, 0, 0, 0, 0, 0, 0, 192]));
    known_values!(boolean "?", (true, [1]), (false, [0]));
}

macro_rules! test_transitiveness {
    ($testname:ident, $modifier:literal, $fmt:literal, $fix:expr) => {
        #[test]
        fn $testname() {
            strct!($modifier $fmt);
            let inp: <Foo as restruct::Struct>::Unpacked = $fix;
            let mut buffer = Vec::new();
            Foo::write_to(inp, &mut buffer).unwrap();
            let outp: <Foo as restruct::Struct>::Unpacked = Foo::read_from(&mut &buffer[..]).unwrap();
            assert_eq!(inp, outp);
        }
    };
    ($modname:ident, $fmt:literal, $fix:expr) => {
        mod $modname {
            test_transitiveness!(none, "", $fmt, $fix);
            test_transitiveness!(native, "@", $fmt, $fix);
            test_transitiveness!(native_std, "=", $fmt, $fix);
            test_transitiveness!(le, "<", $fmt, $fix);
            test_transitiveness!(be, ">", $fmt, $fix);
            test_transitiveness!(network, "!", $fmt, $fix);
        }
    };
    () => {
        mod transitiviness {
            test_transitiveness!(
                signed,
                "bhilqfd?",
                (
                    100,
                    -32000,
                    i32::min_value(),
                    i32::min_value().into(),
                    i64::min_value(),
                    std::f32::consts::PI,
                    std::f64::consts::PI,
                    true
                )
            );
            test_transitiveness!(
                unsigned,
                "BHILQfd?",
                (
                    128,
                    65000,
                    u32::max_value(),
                    u32::max_value().into(),
                    u64::max_value(),
                    std::f32::consts::PI,
                    std::f64::consts::PI,
                    true
                )
            );
        }
    };
}
test_transitiveness!();

#![feature(const_fn_transmute)]

#[macro_use]
mod common;

#[test]
fn io() {
    strct!("<iihf3s2?");

    let fix: <Foo as restruct::Struct>::Packed = [
        0x44, 0x33, 0x22, 0x11, 0x11, 0x22, 0x33, 0x44, 0x66, 0x55, 0xdb, 0x0f, 0x49, 0x40, 0xaa,
        0xbb, 0xcc, 0x01, 0x00,
    ];
    let inp: <Foo as restruct::Struct>::Unpacked = (
        0x1122_3344,
        0x4433_2211,
        0x5566,
        std::f32::consts::PI,
        [0xaa, 0xbb, 0xcc],
        true,
        false,
    );

    let mut buffer = Vec::new();
    Foo::write_to(inp, &mut buffer).unwrap();
    assert_eq!(&buffer[..], &fix[..]);
    let c = Foo::read_from(&mut &buffer[..]).unwrap();
    assert_eq!(c, inp);
}

#[test]
fn unpack_slice() {
    strct!(">hh");
    let buf = vec![0, 1, 0, 2, 255, 255, 255, 255];
    let outp = Foo::unpack_slice(&buf);
    assert_eq!(outp, (1, 2));
}

#[test]
#[should_panic]
fn unpack_slice_panics() {
    strct!(">hh");
    let buf = vec![0, 1, 0];
    Foo::unpack_slice(&buf);
}

#[test]
fn empty_fmt() {
    strct!();
    assert_eq!(Foo::SIZE, 0);
    assert!(Foo::FIELDS.is_empty());
    assert_eq!(Foo::pack(()), [0u8; 0]);
    let _: () = Foo::unpack([0u8; 0]);
}

#[test]
fn zeroed_format_is_zst() {
    strct!("=0b0i");
    assert_eq!(Foo::SIZE, 0);
    assert_eq!(Foo::FIELDS.len(), 2);
    assert_eq!(Foo::pack(()), [0u8; 0]);
    let _: () = Foo::unpack([0u8; 0]);
}

#[test]
fn aligned_to_different_type() {
    strct!(Foo "bb");
    strct!(Bar "bib");
    strct!(Foobar "b0ib");
    if Foobar::SIZE <= Foo::SIZE {
        unreachable!();
    }
    if Foobar::SIZE >= Bar::SIZE {
        unreachable!();
    }
    assert_eq!(Foobar::unpack(Foobar::pack((1, 2))), (1, 2));
}

#[test]
fn tail_aligned() {
    strct!(Foo "b0q");
    strct!(Bar "q");
    let buf = Foo::pack((100,));
    assert_eq!(buf.len(), Foo::SIZE);
    assert_eq!(Foo::SIZE, Bar::SIZE);
    assert_eq!(Foo::unpack(Foo::pack((100,))), (100,));
}

#[test]
fn std_aligned_on_zeroed_type() {
    strct!(Foo "@b0q");
    strct!(Bar "=b0q");
    assert_eq!(Bar::SIZE, 1);
    if Foo::SIZE <= Bar::SIZE {
        unreachable!();
    }
}

#[test]
fn zeroed_type_in_front() {
    strct!(Foo "0ib");
    strct!(Bar "b");
    assert_eq!(Foo::SIZE, Bar::SIZE);
}

#[test]
fn nested_size() {
    strct!(Foo "=i3s");
    strct!(Bar "=2`Foo`");
    assert_eq!(Bar::SIZE, Foo::SIZE * 2);
    Bar::pack(((1, [0, 1, 2]), (2, [10, 20, 30])));
}

#[test]
fn nested() {
    strct!(Foo "Ih");
    strct!(Bar "`Foo`2H");
    let packed = Bar::pack(((3, 592), 43200, 21000));
    let unpacked = Bar::unpack(packed);
    assert_eq!(unpacked, ((3, 592), 43200, 21000));
}

#[test]
fn deeply_nested() {
    strct!(MostBottomTurtle "2?");
    strct!(BottomTurtle "L1`MostBottomTurtle`");
    strct!(Turtle "`BottomTurtle`2s");
    let t = ((999, (true, false)), [1, 2]);
    let packed = Turtle::pack(t);
    let unpacked = Turtle::unpack(packed);
    assert_eq!(unpacked, t);
}

#[test]
fn only_padding() {
    strct!("3x");
    assert_eq!(Foo::SIZE, 3);
    assert_eq!(Foo::pack(()), [0u8; 3]);
}

#[test]
fn padding() {
    strct!("=b2x?");
    strct!(Bar "=b?");
    assert_eq!(Foo::unpack(Foo::pack((-127, false))), (-127, false));
    assert_eq!(Foo::SIZE, Bar::SIZE + 2);
}

macro_rules! test_native_integer_sizes {
    ($testname:ident, $modifier:literal, $fmt1:literal, $fmt2:literal) => {
        #[test]
        fn $testname() {
            strct!(Foo $modifier $fmt1);
            strct!(Bar $modifier $fmt2);
            assert_eq!(Foo::SIZE, Bar::SIZE);
        }
    };
    ($modname:ident, $fmt1:literal, $fmt2:literal) => {
        mod $modname {
            test_native_integer_sizes!(none, "", $fmt1, $fmt2);
            test_native_integer_sizes!(native, "@", $fmt1, $fmt2);
        }
    };
    () => {
        mod native_integer_sizes {
            test_native_integer_sizes!(chr, "b", "B");
            test_native_integer_sizes!(short, "h", "H");
            test_native_integer_sizes!(int, "i", "I");
            test_native_integer_sizes!(long, "l", "L");
            test_native_integer_sizes!(longlong, "q", "Q");
        }
    };
}
test_native_integer_sizes!();

macro_rules! test_relative_sizes {
    ($testname:ident, $modifier:literal, $fmt1:literal, $fmt2:literal) => {
        #[test]
        fn $testname() {
            strct!(Foo $modifier $fmt1);
            strct!(Bar $modifier $fmt2);
            assert!(Foo::SIZE <= Bar::SIZE);
        }
    };
    ($modname:ident, $fmt1:literal, $fmt2:literal) => {
        mod $modname {
            test_relative_sizes!(none, "", $fmt1, $fmt2);
            test_relative_sizes!(native_std, "=", $fmt1, $fmt2);
            test_relative_sizes!(le, "<", $fmt1, $fmt2);
            test_relative_sizes!(be, "<", $fmt1, $fmt2);
            test_relative_sizes!(native, "@", $fmt1, $fmt2);
            test_relative_sizes!(network, "!", $fmt1, $fmt2);
        }
    };
    () => {
        mod native_integer_relative_sizes {
            test_relative_sizes!(short_to_int, "h", "i");
            test_relative_sizes!(int_to_long, "i", "l");
            test_relative_sizes!(long_to_longlong, "l", "q");
        }
    };
}
test_relative_sizes!();

macro_rules! test_absolute_sizes {
    ($testname:ident, $modifier:literal, $fmt:literal, $eq:tt $fix:expr) => {
        #[test]
        fn $testname() {
            strct!($modifier $fmt);
            assert!(Foo::SIZE $eq $fix);
        }
    };
    ($($modname:ident, $fmt:literal, $eq:tt $fix:expr),+) => {
        $(
        mod $modname {
            test_absolute_sizes!(none, "", $fmt, $eq $fix);
            test_absolute_sizes!(native_std, "=", $fmt, $eq $fix);
            test_absolute_sizes!(le, "<", $fmt, $eq $fix);
            test_absolute_sizes!(be, ">", $fmt, $eq $fix);
            test_absolute_sizes!(native, "@", $fmt, $eq $fix);
            test_absolute_sizes!(network, "!", $fmt, $eq $fix);
        }
        )+
    };
    () => {
        mod native_integer_absolute_sizes {
            test_absolute_sizes!(chr, "b", == 1,
                            long, "l", >= 4,
                            longlong, "q", >= 8);
        }
    }
}
test_absolute_sizes!();

macro_rules! test_std_sizes {
    ($testname:ident, $modifier:literal, $fmt:literal, $fix:expr) => {
        #[test]
        fn $testname() {
            strct!($modifier $fmt);
            assert_eq!(Foo::SIZE, $fix);
        }
    };
    ($($modname:ident, $fmt:literal, $fix:literal),+) => {
        $(
        mod $modname {
            test_std_sizes!(native_std, "=", $fmt, $fix);
            test_std_sizes!(le, "<", $fmt, $fix);
            test_std_sizes!(be, ">", $fmt, $fix);
            test_std_sizes!(network, "!", $fmt, $fix);
        }
        )+
    };
    () => {
        mod standard_int_sizes {
            test_std_sizes!(chr, "b", 1,
                            uchr, "B", 1,
                            short, "h", 2,
                            ushort, "H", 2,
                            int, "i", 4,
                            uint, "I", 4,
                            long, "l", 4,
                            ulong, "L", 4,
                            longlong, "q", 8,
                            ulonglong, "Q", 8);
        }
    }
}
test_std_sizes!();

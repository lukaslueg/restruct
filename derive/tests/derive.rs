#![feature(const_fn)]
#![feature(const_fn_transmute)]

#[macro_use]
mod common;

#[test]
fn simple_sizes() {
    strct!(Foo "i");
    strct!(Bar "3i");
    assert_eq!(Foo::SIZE * 3, Bar::SIZE);
}

#[test]
fn default_type_names() {
    strct!();
    type _X = <Foo as restruct::Struct>::Packed;
    type _Y = <Foo as restruct::Struct>::Unpacked;
}

#[test]
fn constness() {
    strct!("ih");
    const FIX: <Foo as restruct::Struct>::Unpacked = (1, 2);
    const SIZE: usize = Foo::SIZE;
    const BUF: <Foo as restruct::Struct>::Packed = Foo::pack(FIX);
    const OUT: <Foo as restruct::Struct>::Unpacked = Foo::unpack(BUF);
    if SIZE == 0 || BUF.len() != Foo::SIZE {
        unreachable!();
    }
    assert_eq!(OUT, FIX);
}

#[test]
fn concatenate_fmt() {
    strct!("<" "2" "i");
    assert_eq!(Foo::SIZE, 8);
}

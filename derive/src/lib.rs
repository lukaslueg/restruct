// MIT License
//
// Copyright (c) 2019 Lukas Lueg (lukas.lueg@gmail.com)
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! `restruct` is used to interpret binary data stored in files or other sources or convert
//! between C structs and Rust types and when using a parser-generator is considered
//! disproportionate.
//! It is a brainchild of [Python's struct-module](https://docs.python.org/3/library/struct.html).
//!
//! The library uses Format Strings as compact descriptions of the binary data and the intended
//! conversion to/from Rust-types. The Format Strings are interpreted at compile-time to
//! generate a type whose functions can be used to convert between unstructured and structured data.
//!
//! ```
//! # #![feature(const_int_conversion)]
//! # #![feature(const_fn)]
//! # #![feature(const_slice_len)]
//! # #![feature(const_transmute)]
//! // Generate a parser in little-endian for two 32bit integers, a float and a bool.
//! #[derive(restruct_derive::Struct)]
//! #[fmt="<2if?"]
//! struct FooParser;
//!
//! // Pack a tuple of two integers, a float an a bool into a [u8; _]-buffer.
//! let packed = FooParser::pack((1, 2, 3.0, false));
//! assert_eq!(packed.len(), FooParser::SIZE);
//! // Packing and unpacking can't fail at runtime.
//! let unpacked = FooParser::unpack(packed);
//! assert_eq!(unpacked, (1, 2, 3.0, false));
//!
//! // Packing and unpacking is const
//! const FOOBAR: [u8; FooParser::SIZE] = FooParser::pack((987, 412, std::f32::consts::PI, false));
//! const BARFOO: <FooParser as restruct::Struct>::Unpacked = FooParser::unpack(FOOBAR);
//! assert_eq!(BARFOO, (987, 412, std::f32::consts::PI, false));
//!
//! // Read/Write data in the given format to a `io::Read/Write`
//! let mut buffer = Vec::new();
//! let inp = (123, 456, -2.521, false);
//! FooParser::write_to(inp, &mut buffer)?;
//! let outp = FooParser::read_from(&mut &buffer[..])?;
//! assert_eq!(outp, inp);
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! ```ignore
//! # #![feature(const_int_conversion)]
//! # #![feature(const_fn)]
//! # #![feature(const_slice_len)]
//! # #![feature(const_transmute)]
//! // As the packing- and unpacking-functions are const, they can initialize other constants.
//! // Read some file from disk and directly unpack it into a const during compilation.
//! #[derive(restruct_derive::Struct)]
//! #[fmt="<2if?"]
//! struct Tea;
//!
//! const TEAPOT: <Tea as restruct::Struct>::Unpacked = Tea::unpack(*include_bytes!("teapot.bin"));
//! const TEAPOT_TEMPERATURE: i32 = TEAPOT.0;
//! const TEAPOT_FILL_STATUS: f32 = TEAPOT.2;
//! const TEAPOT_ACTIVE: bool = TEAPOT.3;
//! ```
//!
//! The type-layout is determined entireley at compile-time. The "packed" representation is always
//! a fixed-length `[u8; ...]`-array. The "unpacked" representation is a (possibly nested) tuple of
//! primitive types.
//!
//! It is not possible to describe variable-sized types like `String` or `Vec<T>` (see the
//! Examples-sections for more information) or to generate/modify parsers at runtime; parsers can be
//! generated via macros (including proc-macros), though.
//!
//! The conversion functions are `const` and may therefor be used in a const-context.
//! As long as endianess does not need to be converted and copying can be elided, packing and
//! unpacking should usually be free of any runtime cost.
//!
//! *Note that this crate is currently nightly-only; the following feature-gates need to be
//! unsealed:*
//! ```
//! #![feature(const_int_conversion)]
//! #![feature(const_fn)]
//! #![feature(const_slice_len)]
//! #![feature(const_transmute)]
//! ```
//!
//! # Deriving
//!
//! Parsers are derived on types using the `Struct`-proc-macro from the `restruct_derive` crate.
//! The Format String is passed via the `fmt`-attribute.
//!
//! ```
//! # #![feature(const_int_conversion)]
//! # #![feature(const_fn)]
//! # #![feature(const_slice_len)]
//! # #![feature(const_transmute)]
//! #[derive(restruct_derive::Struct)]
//! #[fmt=">3Qb2?l"]
//! struct FrameHeader;
//! ```
//!
//! The `fmt`-attribute can be used multiple times and all fragments are concatenated before being
//! interpreted.
//!
//! The proc-macro will add the following items to the given type, among others:
//!
//!  * An implementation of `restruct::Struct`, which will hold the type aliases
//!  for the packed and unpacked representation. For example,
//!  `<Foo as restruct::Struct>::Packed` will be a type alias for `[u8; N]`,
//!  where `N` is some `const`, and `...::Unpacked` will be a tuple.
//!  * An associated constant `SIZE`, which gives the size in bytes of the packed form.
//!  * An associated constant `FIELDS`, an array of tuples of the form
//!  `(&'static str, usize, usize, usize)` for the name of the type, the offset,
//!  the alignment and the total size of each field.
//!  * A `const fn pack()` to convert from unpacked (tuple) into packed (array) form.
//!  * A `const fn unpack()` to convert from packed (array) into unpacked (tuple) form.
//!  * A `fn unpack_slice()` that takes a `&[u8]`-slice and unpacks it's content.
//!    This method will panic if the given slice is smaller than `Self::SIZE`.
//!  * A `fn read_from()` to read one unpacked instance from an any `io::Read`.
//!  * A `fn write_to()` to write one unpacked instance to any `io::Write`.
//!  * A `unsafe fn from_raw<T>(ptr: *const T)` to read one unpacked instance from
//!  a raw pointer.
//!  * An implementation of `std::fmt::Debug`.
//!
//!
//! # Format Strings
//!
//! Format Strings are used to specify the exact byte-layout when packing and unpacking
//! data.
//!
//! The first character in a Format String may be used to control the byte order, size and internal
//! alignment for all following Format Characters. For example, a Format String starting with
//! `"<..."` specifies that all following Format Characters shall be interpreted as little-endian,
//! shall use primitive types and shall not add alignment while packing/unpacking. This can be set
//! only once in a Format String.
//!
//! Zero or more Format Characters may be given to specify the type of data being packed/unpacked.
//! Format Characters map to type aliases defined by the `libc` crate when using native mode (`@`)
//! or primitive types when using standard mode (`=`, `<`, `>` and `!`). For example, `"@l"` refers
//! to `libc::c_long`, which is a type alias for either `i32` or `i64` depending on the current
//! platform; `"=l"` always refers to `i32` and so does `"<l"`, `">l"` and `"!l"`.
//!
//!
//! ## Byte Order, Size, and Alignment
//!
//! | Character   | Byte order             | Size     | Alignment |
//! |-------------|------------------------|----------|-----------|
//! | `@`         | native                 | native   | native    |
//! | `=`         | native                 | standard | none      |
//! | `<`         | little-endian          | standard | none      |
//! | `>`         | big-endian             | standard | none      |
//! | `!`         | network (= big-endian) | standard | none      |
//!
//! If the first character is not one of these, `@` is assumed.
//!
//! Alignment between types is added only in native mode (`@`). For example, the Format String
//! `"@bL"` (usually) describes a `(i8, u64)`, which will result in a `[u8; 16]` when
//! packed: 1 byte for the `i8`, seven alignment bytes and then eight bytes for the `u64`.
//! Alignment is never added at the start or end of the packed data; add a type with a repeat count
//! of zero to add alignment for that type.
//!
//! As a general rule, you should use standard types when dealing with data from IO (e.g.
//! file-formats, protocols, anything persisted and transfered to other platforms, etc.) and native
//! types when reading data structures from memory.
//!
//! ## Format Characters
//!
//!
//! | Format        | Native type         | Standard type |
//! |---------------|---------------------|---------------|
//! | `x`           | _no value_          | _no value_    |
//! | `b`           | `libc::c_char`      | `i8`          |
//! | `B`           | `libc::c_uchar`     | `u8`          |
//! | `?`           | `bool`              | `bool`        |
//! | `h`           | `libc::c_short`     | `i16`         |
//! | `H`           | `libc::c_ushort`    | `u16`         |
//! | `i`           | `libc::c_int`       | `i32`         |
//! | `I`           | `libc::c_uint`      | `u32`         |
//! | `l`           | `libc::c_long`      | `i32`         |
//! | `L`           | `libc::c_ulong`     | `u32`         |
//! | `q`           | `libc::c_longlong`  | `i64`         |
//! | `Q`           | `libc::c_ulonglong` | `u64`         |
//! | `n`           | `libc::ssize_t`     | `isize`       |
//! | `N`           | `libc::size_t`      | `usize`       |
//! | `f`           | `libc::float`       | `f32`         |
//! | `d`           | `libc::double`      | `f64`         |
//! | `s`           | `[u8; _]`           | `[u8; _]`     |
//! | `` `ident` `` | `<ident as restruct::Struct>::Packed` | `<ident as restruct::Struct>::Packed` |
//!
//! A Format Character may be preceded by an repeat count. For example,
//! the format string ``"3x4h2`Foo`"`` means exactly the same as ``"xxx hhhh `Foo` `Foo`"``.
//!
//! Whitespace characters between formats are ignored; a count and its format must not contain
//! whitespace.
//!
//! Native types are indirected via the `libc` crate to Rust's primitive-types. Therefor
//! `libc` must be available in the final crate when using native Format Strings. See the
//! Examples-section for caveats.
//!
//! For the `s` Format Character, the count is interpreted as the length of a `[u8; _]`-array, not
//! a repeat count like for the other format characters. For example, `"3s?"` means `([u8; 3], bool)`
//! while `"3f?"` means `(f32, f32, f32, bool)`.
//!
//! For the `?` Format Character, values not equal to `0` are interpreted as `true` when unpacking.
//! When packing a `bool`, `true` is represented as `1`, `false` as `0`; it's size is alway one
//! byte.
//!
//! The special Format "`` `...` ``" allows to refer to another type which was derived using this crate.
//! In it's packed form, a nested tuple is expected. For example, after deriving a type `Foo`
//! with Format String `"<b2i"`, another Format String on type `Bar` can refer to this as
//! ``"<?2`Foo`"``, resulting in `Bar` expectecting a packed type `(bool, (i8, i32, i32), (i8, i32, i32))`.
//!
//! The `x` Format Character denotes padding bytes. While they contribute to the size of the packed
//! form, they are not present in the unpacked representation. For example, `"?2x?"` will be a
//! `(bool, bool)` in unpacked and a `[u8; 4]` in packed form. Padding bytes are always set to 0
//! in the packed form.
//!
//! The repeat count 0 has special meaning in the sense that the field will not be present in the
//! unpacked representation and only it's alignment contributes to the size of the packed
//! representation (if alignment is considered at all, see above). For example, the Format String
//! `"b0q"` describes a `(i8, )` while the end of the packed representation is aligned to a `i64`;
//! the packed representation is therefor a `[u8; 8]`. Using a repeat count of 0 is effectively a
//! no-op when using Format Strings where alignment is not taken into account.
//!
//! # Examples
//!
//! Packing three integers using standard sizes in big-endian:
//! ```
//! # #![feature(const_int_conversion)]
//! # #![feature(const_fn)]
//! # #![feature(const_slice_len)]
//! # #![feature(const_transmute)]
//! #[derive(restruct_derive::Struct)]
//! #[fmt = ">2hl"]
//! struct Foobar;
//!
//! // The derived type implements std::fmt::Debug
//! dbg!(Foobar);
//!
//! let input = (1, 2, 3);
//! let expected = [0, 1, 0, 2, 0, 0, 0, 3];
//! let packed = Foobar::pack(input);
//! assert_eq!(packed, expected);
//! let unpacked = Foobar::unpack(packed);
//! assert_eq!(unpacked, input);
//! ```
//! ---
//!
//! The Format String can passed in multiple parts, simplifying construction by macros:
//! ```
//! # #![feature(const_int_conversion)]
//! # #![feature(const_fn)]
//! # #![feature(const_slice_len)]
//! # #![feature(const_transmute)]
//! #[derive(restruct_derive::Struct)]
//! #[fmt = ">"]
//! #[fmt = "2h"]
//! #[fmt = "10x"]
//! #[fmt = "3i"]
//! struct Foobar;
//! ```
//! ---
//!
//! Slices can be unpacked at the cost of a copy (which may get elided):
//! ```
//! # #![feature(const_int_conversion)]
//! # #![feature(const_fn)]
//! # #![feature(const_slice_len)]
//! # #![feature(const_transmute)]
//! #[derive(restruct_derive::Struct)]
//! #[fmt = ">2h"]
//! struct Foobar;
//!
//! let buf = vec![0, 1, 0, 2, 255, 255];
//! let unpacked = Foobar::unpack_slice(&buf);
//! assert_eq!(unpacked, (1, 2));
//! ```
//! ---
//!
//! The derived types can be referred to via the `Struct` trait:
//! ```
//! # #![feature(const_int_conversion)]
//! # #![feature(const_fn)]
//! # #![feature(const_slice_len)]
//! # #![feature(const_transmute)]
//! use std::io::{self, Read};
//!
//! #[derive(restruct_derive::Struct)]
//! #[fmt = ">2hl"]
//! struct FoobarHeader;
//!
//! impl FoobarHeader {
//!     pub fn read_header<R>(r: &mut R) -> io::Result<<Self as restruct::Struct>::Unpacked>
//!      where R: io::Read
//!     {
//!         Self::read_from(r)
//!     }
//! }
//! ```
//! ---
//!
//! As packing and unpacking is `const`, these functions can be used to initialize other constants.
//! ```ignore
//! #[derive(restruct_derive::Struct)]
//! #[fmt = ">2hl"]
//! struct Header;
//!
//! const DEFAULT_HEADER: [u8; Header::SIZE] = Header::pack((0x0001, 0xff00, 0xdeadc0de));
//! const HEADER: <Header as restruct::Struct>::Unpacked = Header::unpack(*include_bytes!("header.bin"));
//! ```
//! ---
//!
//! Format Strings always describe fixed-sized data stuctures. When dealing when variable-sized
//! formats, two steps are necessary:
//! ```
//! # #![feature(const_int_conversion)]
//! # #![feature(const_fn)]
//! # #![feature(const_slice_len)]
//! # #![feature(const_transmute)]
//! use std::io::{self, Write};
//!
//! #[derive(restruct_derive::Struct)]
//! #[fmt = "<IN"]
//! struct Frame;
//!
//! impl Frame {
//!     /// Read one frame from the given reader, returning it's payload
//!     pub fn read<R>(r: &mut R) -> io::Result<Vec<u8>>
//!      where R: io::Read {
//!         // Read the fixed-size header
//!         let (magic, size) = Frame::read_from(r)?;
//!         if magic != 0xdeadc0de {
//!             panic!("Unknown frame-format!");
//!         }
//!         // We know the size of the now following data
//!         let mut buf = vec![0; size];
//!         r.read_exact(&mut buf).and(Ok(buf))
//!     }
//!
//!     /// Write the given data as a new frame to the given writer.
//!     pub fn write<W>(buf: &[u8], w: &mut W) -> io::Result<()>
//!      where W: io::Write {
//!         // Write the header
//!         Frame::write_to((0xdeadc0de, buf.len()), w)?;
//!         // ... and the rest
//!         w.write_all(buf)
//!     }
//! }
//!
//! let mut buf = Vec::new();
//! let content = String::from("The quick brown fox");
//! Frame::write(content.as_ref(), &mut buf).expect("Writing failed");
//! // ...
//! let new_buf = Frame::read(&mut buf.as_slice()).expect("Reading failed");
//! let new_content = String::from_utf8(new_buf).expect("UTF-8 decoding failed");
//! assert_eq!(new_content, content);
//! ```
//! ---
//!
//! Alignment rules apply in native mode:
//! ```
//! # #![feature(const_int_conversion)]
//! # #![feature(const_fn)]
//! # #![feature(const_slice_len)]
//! # #![feature(const_transmute)]
//! #[derive(restruct_derive::Struct)]
//! #[fmt = "b0ib"]
//! struct Foobar;
//! // The second `i8` will be aligned to the boundary of an `libc::c_int`, the final
//! // size is therefor larger than two bytes.
//! assert!(Foobar::SIZE > 2);
//!
//! #[derive(restruct_derive::Struct)]
//! #[fmt = "b0Q"]
//! struct Barfoo;
//! // The end of the packed representation is aligned to a `libc::c_ulonglong`, which
//! // means it will be 8 bytes in total.
//! assert_eq!(Barfoo::SIZE, 8);
//! ```
//! ---
//!
//! Formats can refer to previous definitions:
//! ```
//! # #![feature(const_int_conversion)]
//! # #![feature(const_fn)]
//! # #![feature(const_slice_len)]
//! # #![feature(const_transmute)]
//! #[derive(restruct_derive::Struct)]
//! #[fmt = "=i3s"]
//! struct Foo;
//!
//! #[derive(restruct_derive::Struct)]
//! #[fmt = "=2`Foo`"]
//! struct Bar;
//!
//! assert_eq!(Bar::SIZE, Foo::SIZE * 2);
//! assert_eq!(Bar::pack(((1, [0, 1, 2]), (2, [10, 20, 30]))).len(), Bar::SIZE);
//! ```
//! ---
//!
//! When using native types, caveats may appear regarding the actual type used:
//!
//! ```ignore
//! extern "C" {
//!     fn read_header() -> *const u8;
//! }
//!
//! #[derive(restruct_derive::Struct)]
//! #[fmt = "@Lhb"]
//! struct Header;
//!
//! impl Header {
//!     pub fn read_magic() -> u64 {
//!         let header = unsafe { Self::from_raw(read_header()) };
//!         header.0
//!     }
//! }
//! ```
//! The `read_magic()` function is defined to return a `u64`, which needs to match the `"L"` used
//! in the Format String. This will work fine on platforms where `c_ulong` is a `u64` but fail to
//! compile e.g. on i586-platforms where `c_ulong` is a `u32`. Either use `std::convert::TryFrom`
//! or make sure to use the type aliases from `libc` when using native mode.
//!
//! Also note that the `"@...b"` in the Format String above is aliased via `libc::c_char`; it
//! resolves to `i8` on x86-platforms but `u8` on ARM because `c_char` is unsigned on that
//! platform. A line like `header.2 < 0` will - rightfully so - cause a compile-error on ARM.
//! ---
//!
//! When converting from native structs, you must be **sure** that your layout description
//! matches the actual layout:
//! ```ignore
//! # #![feature(const_int_conversion)]
//! # #![feature(const_fn)]
//! # #![feature(const_slice_len)]
//! # #![feature(const_transmute)]
//! #[derive(restruct_derive::Struct)]
//! #[fmt = "@2dl"]
//! struct Header;
//!
//! let head = unsafe { Header::from_raw(...) };
//! ```
//! Let assume that the C-struct we try to match above uses `int32_t` as it's third element.
//! The layout above will match on 32bit-platforms where `"@...l"` is `i32`. On 64bit-platforms
//! however `"@...l"` is `i64`, so `from_raw()` will cause an out-of-bounds memory access by four
//! bytes on those platforms! The correct Format String would have been `"@2di"`.
//! ---

#![feature(external_doc)]

extern crate proc_macro;

/// Derive packing/unpacking on a given type. See the main documentation on this crate for details.
///
/// * Attribute *fmt* gives the Format String.
/// * Attribute *debug_output* causes the generated `TokenStream` to be dumped to stderr while
/// compiling. If the `rustfmt` feature has been activated, the `TokenStream` is formatted.
///
/// Both attributes can appear multiple times. Format Strings are concatenated before being
/// interpreted. The *debug_output* may appear with our without a boolean parameter, with the final
/// occurance being used.
#[proc_macro_derive(Struct, attributes(fmt, debug_output))]
pub fn derive_parser(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    restruct::derive(input.into()).into()
}

#[doc(include = "../README.md")]
#[allow(dead_code)]
type _READMETEST = ();

[![Build Status](https://travis-ci.org/lukaslueg/restruct.svg?branch=master)](https://travis-ci.org/lukaslueg/restruct)
[![Build status](https://ci.appveyor.com/api/projects/status/ufh9cuameeqe3hsx?svg=true)](https://ci.appveyor.com/project/lukaslueg/restruct)


`restruct` is used to interpret binary data stored in files or other sources or convert
between C structs and Rust types and when using a parser-generator is considered
disproportionate.
It is a brainchild of [Python's struct-module](https://docs.python.org/3/library/struct.html).

The library uses Format Strings as compact descriptions of the binary data and the intended
conversion to/from Rust-types. The Format Strings are interpreted at compile-time to
generate a type whose functions can be used to convert between unstructured and structured data.

```rust
#![feature(const_int_conversion)]
#![feature(const_fn)]
#![feature(const_slice_len)]
#![feature(const_transmute)]

// Generate a parser in little-endian for two 32bit integers, a float and a bool.
#[derive(restruct_derive::Struct)]
#[fmt="<2if?"]
struct FooParser;

// Pack a tuple of two integers, a float an a bool into a [u8; _]-buffer.
let packed = FooParser::pack((1, 2, 3.0, false));
assert_eq!(packed.len(), FooParser::SIZE);

// Packing and unpacking can't fail at runtime.
let unpacked = FooParser::unpack(packed);
assert_eq!(unpacked, (1, 2, 3.0, false));
```

```rust,ignore
// Because packing and unpacking is const we can use these functions to initialize other consts
const TEAPOT: <Tea as restruct::Struct>::Unpacked = Tea::unpack(*include_bytes!("teapot.bin"));
const TEAPOT_TEMPERATURE: i32 = TEAPOT.0;
const TEAPOT_FILL_STATUS: f32 = TEAPOT.2;
const TEAPOT_ACTIVE: bool = TEAPOT.3;
```

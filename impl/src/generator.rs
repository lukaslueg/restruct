// The very point of this code-generator is to end up with `const fn pack((...)) -> [u8; _]`
// and `const fn unpack([u8; _]) -> (...)`. This is roughly how it works:
//
// * For every instance of `Format` a `const FIELDx_OFFSET: usize`, `..._ALIGNMENT` and `...SIZE`
// is created.
// * The first field always has it's offset and alignment set to 0; it's size is
// some const-expression which yields the number of bytes required in packed form. For example,
// a `Format::Short` will yield `i16::min_value().to_ne_bytes().len()`, a `Format::Array(5)` will
// yield `5`.
// * All following fields have their offset set to the previous offset plus the previous size; their
// alignment is determined using the generated `const fn align<T>`; their size is their alignment
// plus the const-expression as mentioned above.
// * The size of the array-representation (`Self::SIZE`) is the last field's offset plus it's size.
// * The pack function create one large tuple with {`[0u8; _]` for alignment and the const-expression
// mentioned above (referencing the input)} for every field; the entire tuple is then transmuted
// into a [u8; Self::SIZE]. The unpack function does the same in reverse.
// * When time comes to compile, we let the const-folding-pass do it's job, following the chain of
// consts down to `FIELD0`. If everything adds up, it compiles.

// The big assumption here is that any tuple can be transmuted into a `[u8; _]` as long as their
// sizes match. This does not *have* to be true for all future versions of Rust, as the compiler
// might decide to reorder fields. We probably need one indirection via a repr(C)-struct, which
// should compile to nothing in most cases; the same is true for alignment-calculation.

// Make liberal use of the `rustfmt`-feature and the `#[debug_output]`-attribute.

use crate::parser;
use quote::quote;

#[derive(Clone, Debug)]
enum Format {
    Array(usize),
    Bool,
    Char,
    Double,
    Float,
    Ident(syn::Ident),
    Int,
    Long,
    LongLong,
    Pad(usize),
    Short,
    Size,
    UChar,
    UInt,
    ULong,
    ULongLong,
    UShort,
    USize,
}

impl From<parser::FormatCode> for Format {
    fn from(fc: parser::FormatCode) -> Self {
        use parser::FormatChar::*;
        match fc.chr {
            Array => Format::Array(fc.repeat.unwrap_or(1)),
            Bool => Format::Bool,
            Char => Format::Char,
            Double => Format::Double,
            Float => Format::Float,
            Int => Format::Int,
            Long => Format::Long,
            LongLong => Format::LongLong,
            Pad => Format::Pad(fc.repeat.unwrap_or(1)),
            Short => Format::Short,
            Size => Format::Size,
            UChar => Format::UChar,
            UInt => Format::UInt,
            ULong => Format::ULong,
            ULongLong => Format::ULongLong,
            UShort => Format::UShort,
            USize => Format::USize,
            Ident(ref name) => {
                Format::Ident(syn::Ident::new(&name, proc_macro2::Span::call_site()))
            }
        }
    }
}

#[derive(Debug)]
struct Field {
    ident: syn::Ident,
    fmt: Format,
    materialize: bool,
}

impl Field {
    /// The method to call on numer-types to convert endianess, yielding bytes
    fn to_bytes(order: &parser::ByteOrder) -> syn::Ident {
        syn::Ident::new(
            match order {
                parser::ByteOrder::Native => "to_ne_bytes",
                parser::ByteOrder::LittleEndian => "to_le_bytes",
                parser::ByteOrder::BigEndian => "to_be_bytes",
            },
            proc_macro2::Span::call_site(),
        )
    }

    /// The method to call on numer-types to convert endianess, yielding bytes
    fn from_bytes(order: &parser::ByteOrder) -> syn::Ident {
        syn::Ident::new(
            match order {
                parser::ByteOrder::Native => "from_ne_bytes",
                parser::ByteOrder::LittleEndian => "from_le_bytes",
                parser::ByteOrder::BigEndian => "from_be_bytes",
            },
            proc_macro2::Span::call_site(),
        )
    }

    /// The name of the constant which holds the offset of this fields
    fn offset_ident(&self) -> syn::Ident {
        syn::Ident::new(&format!("{}_OFFSET", self.ident), self.ident.span())
    }

    /// The name of the constant which holds the alignment of this fields
    fn align_ident(&self) -> syn::Ident {
        syn::Ident::new(&format!("{}_ALIGNMENT", self.ident), self.ident.span())
    }

    /// The name of the constant which holds the total size of this fields
    fn size_ident(&self) -> syn::Ident {
        syn::Ident::new(&format!("{}_SIZE", self.ident), self.ident.span())
    }

    /// An (const) expression yielding the size in bytes of this field
    fn size_expr(&self, modifier: &parser::Modifier) -> syn::Expr {
        let tipe = self.tipe(modifier.native_types());
        let tob = Self::to_bytes(&modifier.byte_order());
        match (self.materialize, &self.fmt) {
            (true, Format::Bool) => {
                syn::parse_quote! {
                    1
                }
            }
            (_, Format::Pad(sz)) => {
                syn::parse_quote! {
                    #sz
                }
            }
            (true, Format::Char)
            | (true, Format::UChar)
            | (true, Format::UInt)
            | (true, Format::Int)
            | (true, Format::Size)
            | (true, Format::USize)
            | (true, Format::Long)
            | (true, Format::ULong)
            | (true, Format::LongLong)
            | (true, Format::ULongLong)
            | (true, Format::Short)
            | (true, Format::UShort) => {
                syn::parse_quote! { #tipe::min_value().#tob().len() }
            }
            (true, Format::Float) => {
                syn::parse_quote! {
                    unsafe { std::mem::transmute::<f32, u32>(0.0) }.#tob().len()
                }
            }
            (true, Format::Double) => {
                syn::parse_quote! {
                    unsafe { std::mem::transmute::<f64, u64>(0.0) }.#tob().len()
                }
            }
            (true, Format::Array(sz)) => {
                syn::parse_quote! {
                    #sz
                }
            }
            (true, Format::Ident(ref ident)) => {
                syn::parse_quote! {
                    #ident::SIZE
                }
            }
            (false, _) => {
                syn::parse_quote! { 0 }
            }
        }
    }

    /// The type this field is represented by, e.g. i32 / [u8; 3] / libc::c_uint
    fn tipe(&self, native_types: bool) -> syn::Type {
        match (native_types, &self.fmt) {
            (_, Format::Array(sz)) | (_, Format::Pad(sz)) => syn::parse_quote! { [u8; #sz] },
            (_, Format::Bool) => syn::parse_quote! { bool },
            (_, Format::Ident(ident)) => {
                syn::parse_quote! { <#ident as restruct::Struct>::Unpacked }
            }
            (false, Format::Char) => syn::parse_quote! { i8 },
            (false, Format::Double) => syn::parse_quote! { f64 },
            (false, Format::Float) => syn::parse_quote! { f32 },
            (false, Format::Int) => syn::parse_quote! { i32 },
            (false, Format::Long) => syn::parse_quote! { i32 },
            (false, Format::LongLong) => syn::parse_quote! { i64 },
            (false, Format::Short) => syn::parse_quote! { i16 },
            (false, Format::Size) => syn::parse_quote! { isize },
            (false, Format::UChar) => syn::parse_quote! { u8 },
            (false, Format::UInt) => syn::parse_quote! { u32 },
            (false, Format::ULong) => syn::parse_quote! { u32 },
            (false, Format::ULongLong) => syn::parse_quote! { u64 },
            (false, Format::UShort) => syn::parse_quote! { u16 },
            (false, Format::USize) => syn::parse_quote! { usize },
            (true, Format::Char) => syn::parse_quote! { libc::c_char },
            (true, Format::Double) => syn::parse_quote! { libc::c_double },
            (true, Format::Float) => syn::parse_quote! { libc::c_float },
            (true, Format::Int) => syn::parse_quote! { libc::c_int },
            (true, Format::Long) => syn::parse_quote! { libc::c_long },
            (true, Format::LongLong) => syn::parse_quote! { libc::c_longlong },
            (true, Format::Short) => syn::parse_quote! { libc::c_short },
            (true, Format::Size) => syn::parse_quote! { libc::ssize_t },
            (true, Format::UChar) => syn::parse_quote! {libc::c_uchar },
            (true, Format::UInt) => syn::parse_quote! { libc::c_uint },
            (true, Format::ULong) => syn::parse_quote! { libc::c_ulong },
            (true, Format::ULongLong) => syn::parse_quote! { libc::c_ulonglong },
            (true, Format::UShort) => syn::parse_quote! { libc::c_ushort },
            (true, Format::USize) => syn::parse_quote! { libc::size_t },
        }
    }

    /// A (const) expression yielding the array-representation
    fn pack_expr(&self, modifier: &parser::Modifier, access: &syn::Expr) -> syn::Expr {
        let tob = Self::to_bytes(&modifier.byte_order());
        let tipe = self.tipe(modifier.native_types());
        match self.fmt {
            Format::Bool => {
                syn::parse_quote! {
                    #access as #tipe
                }
            }
            Format::Pad(sz) => {
                syn::parse_quote! {
                    [0u8; #sz]
                }
            }
            Format::Char
            | Format::UChar
            | Format::UInt
            | Format::Int
            | Format::Long
            | Format::Size
            | Format::USize
            | Format::ULong
            | Format::LongLong
            | Format::ULongLong
            | Format::Short
            | Format::UShort => {
                syn::parse_quote! { #access.#tob() }
            }
            Format::Float => {
                // TODO This special handling for f32/f64 can go away once .to_bits() arrives in
                // const-land
                syn::parse_quote! {
                    unsafe { std::mem::transmute::<f32, u32>(#access) }.#tob()
                }
            }
            Format::Double => {
                syn::parse_quote! {
                    unsafe { std::mem::transmute::<f64, u64>(#access) }.#tob()
                }
            }
            Format::Array(_) => access.clone(),
            Format::Ident(ref ident) => {
                syn::parse_quote! {
                    #ident::pack(#access)
                }
            }
        }
    }

    /// A (const) expression yielding the primitive-type representation
    fn unpack_expr(&self, modifier: &parser::Modifier, access: &syn::Expr) -> syn::Expr {
        let tipe = self.tipe(modifier.native_types());
        let fob = Self::from_bytes(&modifier.byte_order());
        match self.fmt {
            Format::Bool => {
                syn::parse_quote! {
                    #access[0] != 0
                }
            }
            Format::Pad(sz) => {
                syn::parse_quote! {
                    [0u8; #sz]
                }
            }
            Format::Char
            | Format::UChar
            | Format::Int
            | Format::UInt
            | Format::Size
            | Format::USize
            | Format::Long
            | Format::ULong
            | Format::LongLong
            | Format::ULongLong
            | Format::Short
            | Format::UShort => {
                syn::parse_quote! { #tipe::#fob(#access) }
            }
            Format::Float => {
                syn::parse_quote! {
                    unsafe { std::mem::transmute(u32::#fob(#access)) }
                }
            }
            Format::Double => {
                syn::parse_quote! {
                    unsafe { std::mem::transmute(u64::#fob(#access)) }
                }
            }
            Format::Array(_) => access.clone(),
            Format::Ident(ref ident) => {
                syn::parse_quote! {
                    #ident::unpack(#access)
                }
            }
        }
    }

    /// A tuple-expression containing the name of the type, the offset, the alignment and the total
    /// size for this field
    fn fields_ary_entry(&self, modifier: &parser::Modifier) -> syn::Expr {
        let o_id = self.offset_ident();
        let a_id = self.align_ident();
        let s_id = self.size_ident();
        let tipe = self.tipe(modifier.native_types());
        syn::parse_quote! {
            (stringify!(#tipe), Self::#o_id, Self::#a_id, Self::#s_id)
        }
    }
}

#[derive(Debug)]
pub struct Compilation {
    name: proc_macro2::Ident,
    fields: Vec<Field>,
    modifier: parser::Modifier,
    generics: syn::Generics,
}

impl Compilation {
    pub fn new(name: proc_macro2::Ident, generics: syn::Generics, format: parser::Format) -> Self {
        let mut fields = Vec::new();
        let modifier = format.modifier.unwrap_or_default();
        let mut fieldcounter =
            (0..).map(|i| syn::Ident::new(&format!("FIELD{}", i), proc_macro2::Span::call_site()));
        for fc in format.codes {
            let repeat = fc.repeat.unwrap_or(1);
            let fmt = Format::from(fc);
            let materialize = match (&fmt, repeat) {
                (Format::Pad(_), _) | (_, 0) => false,
                (_, _) => true,
            };
            match fmt {
                fmt @ Format::Array(_) | fmt @ Format::Pad(_) => {
                    let f = Field {
                        ident: fieldcounter.next().unwrap(),
                        fmt,
                        materialize,
                    };
                    fields.push(f);
                }
                fmt => {
                    for _ in 0..std::cmp::max(1, repeat) {
                        let f = Field {
                            ident: fieldcounter.next().unwrap(),
                            fmt: fmt.clone(),
                            materialize,
                        };
                        fields.push(f);
                    }
                }
            }
        }
        Self {
            name,
            modifier,
            fields,
            generics,
        }
    }

    fn materialized_fields(&self) -> impl Iterator<Item = (usize, &Field)> {
        self.fields
            .iter()
            .enumerate()
            .filter_map(|(i, f)| if f.materialize { Some((i, f)) } else { None })
    }

    fn packed_type(&self) -> syn::Type {
        let name = &self.name;
        syn::parse_quote! {
            [u8; #name::SIZE]
        }
    }

    fn unpacked_type(&self) -> syn::Type {
        let types = self
            .materialized_fields()
            .map(|(_, f)| f.tipe(self.modifier.native_types()));
        syn::parse_quote! {
            (#(#types ,)*)
        }
    }

    /// The unpack method, going from array to tuple
    fn unpack(&self) -> syn::ItemFn {
        let fieldbuffers = (0..self.fields.len()).map(|i| {
            quote! {
                [u8; Self::FIELDS[#i].2], [u8; Self::FIELDS[#i].3 - Self::FIELDS[#i].2]
            }
        });
        let fieldbuffer: syn::TypeTuple = syn::parse_quote! {
            (#(#fieldbuffers),*)
        };
        let fieldvalues = self.materialized_fields().map(|(i, f)| {
            let m = syn::Member::Unnamed((i * 2 + 1).into());
            f.unpack_expr(&self.modifier, &syn::parse_quote! { __STRUCT.#m })
        });
        syn::parse_quote! {
            pub const fn unpack(inp: <Self as restruct::Struct>::Packed) -> <Self as restruct::Struct>::Unpacked {
                let __STRUCT: #fieldbuffer;
                __STRUCT = unsafe { std::mem::transmute(inp) };
                (#(#fieldvalues ,)*)
            }
        }
    }

    /// The pack-method, going from tuple to array
    fn pack(&self) -> syn::ItemFn {
        let mut mi: usize = 0;
        let exprs = self.fields.iter().enumerate().map(|(i, f)| {
            let pack_expr = if f.materialize {
                let m = syn::Member::Unnamed(mi.into());
                let e = f.pack_expr(&self.modifier, &syn::parse_quote! {inp.#m});
                mi += 1;
                e
            } else {
                syn::parse_quote! {
                    [0u8; Self::FIELDS[#i].3 - Self::FIELDS[#i].2]
                }
            };
            quote! {
                [0u8; Self::FIELDS[#i].2], #pack_expr
            }
        });
        syn::parse_quote! {
            pub const fn pack(inp: <Self as restruct::Struct>::Unpacked) -> <Self as restruct::Struct>::Packed {
                let __STRUCT = (#(#exprs, )*);
                unsafe { std::mem::transmute(__STRUCT) }
            }
        }
    }

    /// The `const SIZE: usize`-item resolving to the size of the packed buffer
    fn size(&self) -> syn::ItemConst {
        let size: syn::Expr = self.fields.last().map_or_else(
            || syn::parse_quote! { 0 },
            |f| {
                let o_id = f.offset_ident();
                let s_id = f.size_ident();
                syn::parse_quote! {
                    Self::#o_id + Self::#s_id
                }
            },
        );
        syn::parse_quote! {
            pub const SIZE: usize = #size;
        }
    }

    /// The `const FIELDS: [...; n]`-item holding tuples describing each field
    fn fields_array(&self) -> syn::ItemConst {
        let elemens = self
            .fields
            .iter()
            .map(|f| f.fields_ary_entry(&self.modifier));
        let size = self.fields.len();
        syn::parse_quote! {
            pub const FIELDS: [(&'static str, usize, usize, usize); #size] = [#(#elemens),*];
        }
    }

    /// Generate all the required constants which will resolve to the offset, the alignment
    /// and the total size of each field.
    fn fields(&self) -> Vec<syn::ItemConst> {
        let mut res = Vec::with_capacity(self.fields.len() * 3);

        macro_rules! push {
            ($name:tt, $($tt:tt)*) => {
                res.push(syn::parse_quote!{
                    const #$name: usize = $($tt)*;
                })
            }
        };

        if let Some(first_field) = self.fields.first() {
            let o_id = first_field.offset_ident();
            let a_id = first_field.align_ident();
            let s_id = first_field.size_ident();
            let s_expr = first_field.size_expr(&self.modifier);
            push!(o_id, 0);
            push!(a_id, 0);
            push!(s_id, Self::#a_id + #s_expr);
        }

        for (prev, cur) in self.fields.iter().zip(self.fields.iter().skip(1)) {
            let po_id = prev.offset_ident();
            let ps_id = prev.size_ident();
            let co_id = cur.offset_ident();
            let ca_id = cur.align_ident();
            let cs_id = cur.size_ident();
            let cs_expr = cur.size_expr(&self.modifier);
            let tipe = cur.tipe(self.modifier.native_types());

            push!(co_id, Self::#po_id + Self::#ps_id);

            if self.modifier.native_types() {
                push!(ca_id, Self::align::<#tipe>(Self::#co_id));
            } else {
                push!(ca_id, 0);
            }

            push!(cs_id, Self::#ca_id + #cs_expr);
        }

        res
    }

    /// A Debug-impl
    fn debug_impl(
        &self,
        impl_generics: &syn::ImplGenerics,
        ty_generics: &syn::TypeGenerics,
        where_clause: Option<&syn::WhereClause>,
    ) -> syn::ItemImpl {
        let name = &self.name;
        syn::parse_quote! {
            impl #impl_generics std::fmt::Debug for #name #ty_generics #where_clause {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
                    write!(f, "{} {{", stringify!(#name))?;
                    for (i, e) in Self::FIELDS.iter().enumerate() {
                        write!(f, "(Field {}, type {}, offset {}, alignment {}, size {})", i, e.0, e.1, e.2, e.3)?;
                        if i != Self::FIELDS.len() {
                            write!(f, ", ")?
                        }
                    }
                    write!(f, ", total size {} }}", Self::SIZE)
                }
            }
        }
    }
}

impl quote::ToTokens for Compilation {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let name = &self.name;
        let fields = self.fields();
        let size = self.size();
        let fields_ary = self.fields_array();
        let packed_type = self.packed_type();
        let unpacked_type = self.unpacked_type();
        let pack_fn = self.pack();
        let unpack_fn = self.unpack();
        let debug_impl = self.debug_impl(&impl_generics, &ty_generics, where_clause);

        let res = quote! {
            impl restruct::Struct for #name {
                type Packed = #packed_type;
                type Unpacked = #unpacked_type;
            }

            #[allow(clippy::transmute_int_to_float)]
            impl #impl_generics #name #ty_generics #where_clause {
                const fn align<T>(ptr: usize) -> usize {
                    let align = std::mem::align_of::<T>();
                    let offset = ptr % align;
                    (offset != 0) as usize * (align - offset)
                }
                #(#fields)*
                #size
                #fields_ary
                #pack_fn
                #unpack_fn

                /// Unpack the bytes from the given slice.
                ///
                /// # Panics
                ///
                /// The function will panic if the slice is smaller than `Self::SIZE`
                pub fn unpack_slice(inp: &[u8]) -> <Self as restruct::Struct>::Unpacked {
                    let mut __BUFFER = [0u8; Self::SIZE];
                    __BUFFER.copy_from_slice(&inp[..Self::SIZE]);
                    Self::unpack(__BUFFER)
                }

                /// Pack the given input and write it directly to the given writer.
                pub fn write_to<T: std::io::Write>(inp: <Self as restruct::Struct>::Unpacked, w: &mut T) -> std::io::Result<()> {
                    w.write_all(&Self::pack(inp))
                }

                /// Read exactly `Self::SIZE` bytes from the given reader and unpack them.
                pub fn read_from<T: std::io::Read>(r: &mut T) -> std::io::Result<<Self as restruct::Struct>::Unpacked> {
                    let mut __BUFFER = [0; Self::SIZE];
                    r.read_exact(&mut __BUFFER)?;
                    Ok(Self::unpack(__BUFFER))
                }

                /// Act as if the input was pointing to an `[u8; Self::SIZE]`-array and unpack it
                pub unsafe fn from_raw<T>(ptr: *const T) -> <Self as restruct::Struct>::Unpacked {
                    let ptr = ptr as *const [u8; Self::SIZE];
                    Self::unpack(*ptr)
                }
            }
            #debug_impl
        };
        use quote::TokenStreamExt;
        tokens.append_all(res);
    }
}

impl std::string::ToString for Compilation {
    #[cfg(feature = "rustfmt")]
    fn to_string(&self) -> String {
        use quote::ToTokens;
        let txt = self.into_token_stream().to_string();
        let mut cfg = rustfmt_nightly::Config::default();
        cfg.override_value("emit_mode", "stdout");
        let mut buf = Vec::new();
        {
            let mut session = rustfmt_nightly::Session::new(cfg, Some(&mut buf));
            session.format(rustfmt_nightly::Input::Text(txt)).unwrap();
        }
        String::from_utf8(buf).unwrap()
    }

    #[cfg(not(feature = "rustfmt"))]
    fn to_string(&self) -> String {
        use quote::ToTokens;
        self.into_token_stream().to_string()
    }
}

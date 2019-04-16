#[test]
#[should_panic(expected = "must be a string")]
fn fmt_not_a_string() {
    let tokens = quote::quote! {
        #[fmt=true]
        struct Foo;
    };
    restruct::derive(tokens);
}

#[test]
#[should_panic(expected = "must be a bool")]
fn fmt_attr_not_bool() {
    let tokens = quote::quote! {
        #[fmt="iii"]
        #[debug_output=123]
        struct Foo;
    };
    restruct::derive(tokens);
}

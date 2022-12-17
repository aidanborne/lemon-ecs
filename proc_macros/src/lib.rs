use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    token::Comma,
    *,
};

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let gen = quote! {
        impl lemon_ecs::Component for #name { }
    };

    gen.into()
}

struct ForTuplesInput {
    ident: Ident,
    start: usize,
    end: usize,
}

impl Parse for ForTuplesInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        input.parse::<Comma>()?;
        let start = input.parse::<LitInt>()?.base10_parse::<usize>()?;
        input.parse::<Comma>()?;
        let end = input.parse::<LitInt>()?.base10_parse::<usize>()?;
        Ok(ForTuplesInput { ident, start, end })
    }
}

#[proc_macro]
pub fn for_tuples(input: TokenStream) -> TokenStream {
    let tuples = parse_macro_input!(input as ForTuplesInput);
    let macro_ident = tuples.ident;

    let type_idents: Vec<Ident> = (tuples.start..tuples.end)
        .map(|i| format_ident!("{}{}", "T", i))
        .collect();

    let mut tokens = TokenStream::new();

    for i in tuples.start..(tuples.end - 1) {
        let type_idents = &type_idents[0..i];

        let ast = quote! {
            #macro_ident!(#(#type_idents),*);
        };

        tokens.extend::<TokenStream>(ast.into());
    }

    tokens
}

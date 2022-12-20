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
        impl lemon_ecs::component::Component for #name {
            fn create_storage(&self) -> Box<dyn lemon_ecs::storage::components::ComponentVec> {
                Box::new(Vec::<#name>::new())
            }

            fn component_id(&self) -> std::any::TypeId {
                std::any::TypeId::of::<#name>()
            }
        }
    };

    gen.into()
}

struct TupleRanges {
    ident: Ident,
    start: usize,
    end: usize,
}

impl Parse for TupleRanges {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        input.parse::<Comma>()?;

        let start = input.parse::<LitInt>()?.base10_parse::<usize>()?;
        input.parse::<Token![..]>()?;

        let end = input.parse::<LitInt>()?.base10_parse::<usize>()?;
        Ok(TupleRanges { ident, start, end })
    }
}

#[proc_macro]
pub fn all_tuples(input: TokenStream) -> TokenStream {
    let tuples = parse_macro_input!(input as TupleRanges);
    let macro_name = tuples.ident;

    let idents: Vec<Ident> = (0..tuples.end)
        .map(|i| format_ident!("T{}", i + 1))
        .collect();

    let mut tokens = TokenStream::new();

    for i in tuples.start..tuples.end {
        let idents = &idents[0..i];

        let ast = quote! {
            #macro_name!(#(#idents),*);
        };

        tokens.extend::<TokenStream>(ast.into());
    }

    tokens
}

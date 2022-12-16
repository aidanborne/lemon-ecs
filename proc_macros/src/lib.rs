use proc_macro::{TokenStream};
use quote::{quote, format_ident};
use syn::{*, parse::{Parse, ParseStream}, token::Comma};

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let gen = quote! {
        impl Component for #name { }
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

    let type_idents: Vec<Ident> = (tuples.start..tuples.end).map(|i| format_ident!("{}{}", "T", i)).collect();

    let mut tokens = TokenStream::new();

    for i in tuples.start..(tuples.end - 1) {
        let type_idents = &type_idents[0..i];

        let ast = quote!{
            #macro_ident!(#(#type_idents),*);
        };
       
        tokens.extend::<TokenStream>(ast.into());
    }

    tokens   
}

struct QueryInput {
    idents: Vec<Ident>,
}

impl Parse for QueryInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut idents = Vec::new();
        while !input.is_empty() {
            idents.push(input.parse::<Ident>()?);
            if !input.is_empty() {
                input.parse::<Comma>()?;
            }
        }
        Ok(QueryInput { idents })
    }
}

#[proc_macro]
pub fn impl_query(input: TokenStream) -> TokenStream {
    let query = parse_macro_input!(input as QueryInput);
    let idents = query.idents;

    let ast = quote!{
        impl<'a, #(#idents: 'static + Queryable<'a>),*> Queryable<'a> for (#(#idents),*) {
            type Item = (#(#idents::Item),*);

            fn get_query() -> Query {
                Query::new(vec![#(TypeId::of::<#idents>()),*])
            }

            fn map_entity(archetype: &'a Archetype, id: usize) -> Self::Item {
                (#(#idents::map_entity(archetype, id)),*)
            }
        }
    };
    
    ast.into()
}
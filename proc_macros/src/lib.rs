use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};

use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    *,
};

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = input.ident;

    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let gen = quote! {
        impl #impl_generics lemon_ecs::component::Component for #ident #ty_generics #where_clause {
            #[inline]
            fn as_empty_vec(&self) -> Box<dyn lemon_ecs::component::ComponentVec> {
                Box::new(Vec::<#ident #ty_generics>::new())
            }
        }
    };

    gen.into()
}

// A range with literal start and end values
struct TupleRange {
    start: usize,
    end: usize,
}

impl Parse for TupleRange {
    fn parse(input: ParseStream) -> Result<Self> {
        let start = input.parse::<LitInt>()?.base10_parse::<usize>()?;
        input.parse::<Token![..]>()?;

        let end = input.parse::<LitInt>()?.base10_parse::<usize>()?;
        Ok(TupleRange { start, end })
    }
}

struct AllTuplesInput {
    ident: Ident,
    range: TupleRange,
}

impl Parse for AllTuplesInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;

        let range = input.parse::<TupleRange>()?;
        Ok(AllTuplesInput { ident, range })
    }
}

#[proc_macro]
pub fn all_tuples(input: TokenStream) -> TokenStream {
    let AllTuplesInput { ident, range } = parse_macro_input!(input as AllTuplesInput);

    let idents: Vec<Ident> = (0..range.end)
        .map(|i| format_ident!("T{}", i + 1))
        .collect();

    let mut tokens = TokenStream::new();

    for i in range.start..range.end {
        let idents = &idents[0..i];

        let ast = quote! {
            #ident!(#(#idents),*);
        };

        tokens.extend::<TokenStream>(ast.into());
    }

    tokens
}

#[proc_macro]
pub fn impl_tuple_bundles(input: TokenStream) -> TokenStream {
    let range = parse_macro_input!(input as TupleRange);

    let indices: Vec<Index> = (0..range.end).map(Index::from).collect();

    let idents: Vec<Ident> = (0..range.end)
        .map(|i| format_ident!("T{}", i + 1))
        .collect();

    let mut tokens = TokenStream::new();

    for i in range.start..range.end {
        let idents = &idents[0..i];
        let indices = &indices[0..i];

        let ast = quote! {
            impl<#(#idents),*> Bundle for (#(#idents,)*)
            where
                #(#idents: 'static + Component),*
            {
                fn components(self) -> Vec<Box<dyn Component>> {
                    vec![#(Box::new(self.#indices)),*]
                }
            }
        };

        tokens.extend::<TokenStream>(ast.into());
    }

    tokens
}

#[inline]
fn is_bundle(field: &Field) -> bool {
    field.attrs.iter().any(|attr| attr.path.is_ident("bundle"))
}

fn impl_bundle<T: ToTokens>(
    generics: Generics,
    ident: Ident,
    fields: Punctuated<Field, Token![,]>,
    f: fn(usize, &Field) -> T,
) -> TokenStream {
    let mut components = Vec::<T>::new();
    let mut bundles = Vec::<T>::new();
    let mut types = Vec::new();

    for (idx, field) in fields.iter().enumerate() {
        if is_bundle(field) {
            bundles.push(f(idx, field));
        } else {
            components.push(f(idx, field));
        }

        types.push(&field.ty);
    }

    let gen = quote! {
        impl #generics lemon_ecs::component::Bundle for #ident #generics
        where
            #(#types: 'static + lemon_ecs::component::Bundle),*
        {
            fn components(self) -> Vec<Box<dyn lemon_ecs::component::Component>> {
                let mut components: Vec<Box<dyn lemon_ecs::component::Component>> = vec![#(Box::new(self.#components)),*];
                #(
                    components.extend(self.#bundles.into());
                )*
                components
            }
        }
    };

    gen.into()
}

#[proc_macro_derive(Bundle)]
pub fn derive_bundle(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let fields = match ast.data {
        Data::Struct(DataStruct { fields, .. }) => fields,
        _ => panic!("Only non-unit structs can derive Bundle"),
    };

    match fields {
        Fields::Named(fields) => impl_bundle(ast.generics, ast.ident, fields.named, |_, field| {
            field.ident.clone().unwrap()
        }),
        Fields::Unnamed(fields) => {
            impl_bundle(ast.generics, ast.ident, fields.unnamed, |idx, _| {
                Index::from(idx)
            })
        }
        Fields::Unit => panic!("Unit structs cannot derive Bundle"),
    }
}

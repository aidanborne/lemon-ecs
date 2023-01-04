use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    *,
};

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let gen = quote! {
        impl lemon_ecs::component::Component for #name {
            #[inline]
            fn get_storage(&self) -> Box<dyn lemon_ecs::storage::components::ComponentVec> {
                Box::new(Vec::<#name>::new())
            }
        }
    };

    gen.into()
}

// A range with literal start and end values
struct RangeLiteral {
    start: usize,
    end: usize,
}

impl Parse for RangeLiteral {
    fn parse(input: ParseStream) -> Result<Self> {
        let start = input.parse::<LitInt>()?.base10_parse::<usize>()?;
        input.parse::<Token![..]>()?;

        let end = input.parse::<LitInt>()?.base10_parse::<usize>()?;
        Ok(RangeLiteral { start, end })
    }
}

struct MacroTuples {
    ident: Ident,
    range: RangeLiteral,
}

impl Parse for MacroTuples {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;

        let range = input.parse::<RangeLiteral>()?;
        Ok(MacroTuples { ident, range })
    }
}

#[proc_macro]
pub fn all_tuples(input: TokenStream) -> TokenStream {
    let tuples = parse_macro_input!(input as MacroTuples);
    let macro_ident = tuples.ident;

    let idents: Vec<Ident> = (0..tuples.range.end)
        .map(|i| format_ident!("T{}", i + 1))
        .collect();

    let mut tokens = TokenStream::new();

    for i in tuples.range.start..tuples.range.end {
        let idents = &idents[0..i];

        let ast = quote! {
            #macro_ident!(#(#idents),*);
        };

        tokens.extend::<TokenStream>(ast.into());
    }

    tokens
}

#[proc_macro]
pub fn impl_tuple_bundle(input: TokenStream) -> TokenStream {
    let range = parse_macro_input!(input as RangeLiteral);

    let indices: Vec<Index> = (0..range.end).map(Index::from).collect();

    let idents: Vec<Ident> = (0..range.end)
        .map(|i| format_ident!("T{}", i + 1))
        .collect();

    let mut tokens = TokenStream::new();

    for i in range.start..range.end {
        let idents = &idents[0..i];
        let indices = &indices[0..i];

        let ast = quote! {
            impl<#(#idents),*> Bundleable for (#(#idents,)*)
            where
                #(#idents: 'static + Component),*
            {
                fn bundle(self) -> Vec<Box<dyn Component>> {
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
    field
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("bundle"))
        .is_some()
}

fn derive_into_bundle_helper<T: ToTokens>(
    generics: Generics,
    ident: Ident,
    fields: Punctuated<Field, Token![,]>,
    f: fn(usize, &Field) -> T,
) -> TokenStream {
    let mut components = Vec::<T>::new();
    let mut bundles = Vec::<T>::new();
    let mut types = Vec::new();

    for (idx, field) in fields.iter().enumerate() {
        if is_bundle(&field) {
            bundles.push(f(idx, &field));
        } else {
            components.push(f(idx, &field));
        }

        types.push(&field.ty);
    }

    let gen = quote! {
        impl #generics lemon_ecs::component::bundle::Bundleable for #ident #generics
        where
            #(#types: 'static + lemon_ecs::component::bundle::Bundleable),*
        {
            fn bundle(self) -> lemon_ecs::component::bundle::ComponentBundle {
                let mut bundle: lemon_ecs::component::bundle::ComponentBundle = vec![];
                #(
                    bundle.push(Box::new(self.#components));
                )*
                #(
                    bundle.extend(self.#bundles.into());
                )*
                bundle
            }
        }
    };

    gen.into()
}

#[proc_macro_derive(Bundleable)]
pub fn derive_into_bundle(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let fields = match ast.data {
        Data::Struct(DataStruct { fields, .. }) => fields,
        _ => panic!("Only non-unit structs can derive Bundable"),
    };

    match fields {
        Fields::Named(fields) => {
            derive_into_bundle_helper(ast.generics, ast.ident, fields.named, |_, field| {
                field.ident.clone().unwrap()
            })
        }
        Fields::Unnamed(fields) => {
            derive_into_bundle_helper(ast.generics, ast.ident, fields.unnamed, |idx, _| {
                Index::from(idx)
            })
        }
        Fields::Unit => panic!("Unit structs cannot derive Bundable"),
    }
}

use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use syn::{parse2, parse_quote, Data, DeriveInput, Expr, Type};

#[proc_macro_derive(Layout)]
pub fn layout(input: TokenStream) -> TokenStream {
    struct_layout(input.into()).into()
}

fn struct_layout(input: TokenStream2) -> TokenStream2 {
    let ast: DeriveInput = parse2(input).unwrap();
    let ident = ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let layout_struct = format_ident!("{}Layout", ident);
    let mut layout_ident: Vec<Ident> = Vec::new();
    let mut layout_offset_after: Vec<Expr> = Vec::new();
    let mut layout_offset_prev: Vec<Expr> = Vec::new();
    layout_offset_prev.push(parse_quote! {0});

    if let Data::Struct(data) = ast.data {
        data.fields
            .iter()
            .filter(|x| x.ty.to_token_stream().to_string().contains("PhantomData") == false)
            .for_each(|x| {
                // println!("{}", x.to_token_stream());
                let ty: Type = x.ty.clone();
                let current_size: Expr = parse_quote! {core::mem::size_of::<#ty>()};
                let current_offset: Expr;

                match layout_offset_after.last() {
                    Some(lo) => {
                        current_offset = parse_quote! { #lo + #current_size};
                    }
                    None => {
                        current_offset = parse_quote! {#current_size};
                    }
                }
                layout_offset_after.push(current_offset);
                layout_ident.push(x.clone().ident.expect("Should be all named field"));
            })
    } else {
        panic!("We need struct");
    }

    layout_offset_after
        .iter()
        .for_each(|x| layout_offset_prev.push(x.clone()));
    layout_offset_prev.pop();
    let usage_msg = format!(
        "用于获取 {} 每个字段的内存布局，如 `{}::{}();`",
        ident.to_string(),
        layout_struct.to_string(),
        layout_ident[0].to_string()
    );
    let mut phantom_ident: Vec<Ident> = Vec::new();
    let mut phantom_generic: Vec<Ident> = Vec::new();

    ast.generics.type_params().enumerate().for_each(|x| {
        phantom_ident.push(format_ident!("p{}", x.0));
        phantom_generic.push(x.1.ident.to_owned());
    });
    quote! {
        #[doc=#usage_msg]
        pub struct #layout_struct #ty_generics { #(#phantom_ident: core::marker::PhantomData<#phantom_generic>),*}

        impl #impl_generics #layout_struct #ty_generics #where_clause {
            #(pub  fn #layout_ident()->core::ops::Range<usize> { #layout_offset_prev..#layout_offset_after})*
        }
    }
}

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Path, parse_macro_input};

mod proxy_micro;

// 定义派生宏入口，用法：#[proxy(TraitName)]
#[proc_macro_attribute]
pub fn proxy(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 解析要实现的 trait 路径
    let trait_path = parse_macro_input!(attr as Path);

    // 解析结构体定义
    let derive_input = parse_macro_input!(item as DeriveInput);
    let struct_ident = &derive_input.ident;
    let (impl_generics, ty_generics, where_clause) = derive_input.generics.split_for_impl();

    // 确保是结构体并且有 named fields
    let fields = match &derive_input.data {
        Data::Struct(data_struct) => &data_struct.fields,
        _ => panic!("proxy macro must be applied to a struct with a inner field"),
    };

    let fields_named = match fields {
        Fields::Named(named) => named,
        _ => panic!("proxy macro must be applied to a struct with a inner field"),
    };

    // 查找 inner 字段
    let _inner_field = fields_named
        .named
        .iter()
        .find(|f| f.ident.as_ref().map(|i| i == "inner").unwrap_or(false))
        .unwrap_or_else(|| panic!("proxy macro must be applied to a struct with a inner field"));

    // let inner_ty = &inner_field.ty; // inner的类型（如S0）

    // 4. 生成Trait的方法转发实现（核心逻辑）
    let trait_methods = proxy_micro::generate_trait_methods(&trait_path);

    // 5. 拼接最终生成的代码（为结构体实现目标Trait）
    let expanded = quote! {
        #derive_input

        // 自动为结构体实现目标Trait
        impl #impl_generics #trait_path for #struct_ident #ty_generics #where_clause {
            #(#trait_methods)*
        }
    };

    TokenStream::from(expanded)
}

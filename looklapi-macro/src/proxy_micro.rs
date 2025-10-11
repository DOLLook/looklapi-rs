use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    token::PathSep,
    Attribute, FnArg, Ident, ItemImpl, ItemStruct, MethodSig, Path, PathArguments, PathSegment,
    ReturnType, Token, TraitItem, TraitItemMethod, Type, Visibility,
};

// 解析 #[proxy_for(TraitName)] 属性
struct ProxyForAttr {
    trait_path: Path,
}

impl Parse for ProxyForAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let trait_path = input.parse()?;
        Ok(ProxyForAttr { trait_path })
    }
}

// 解析 #[proxy] 属性
struct ProxyAttr;

impl Parse for ProxyAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(ProxyAttr)
    }
}

// 查找 trait 定义中的所有方法
fn find_trait_methods(trait_path: &Path) -> syn::Result<Vec<TraitItemMethod>> {
    // 在实际项目中，需要从 crate 中查找 trait 定义
    // 这里使用 syn 解析一个虚拟的 trait 作为示例
    // 生产环境中应使用 cargo_metadata 或类似工具查找实际的 trait 定义

    // 注意：这是一个简化实现，实际使用时需要替换为真实的 trait 解析逻辑
    // 可参考 syn 文档实现从源代码解析 trait 定义
    let dummy_trait = format!(
        "pub trait {} {{}}",
        trait_path.segments.last().unwrap().ident
    );
    let ast = syn::parse_str::<syn::File>(&dummy_trait)?;
    
    let mut methods = Vec::new();
    for item in ast.items {
        if let syn::Item::Trait(trait_item) = item {
            for item in trait_item.items {
                if let TraitItem::Method(method) = item {
                    methods.push(method);
                }
            }
        }
    }
    
    Ok(methods)
}

// 获取方法是否有 #[proxy] 属性
fn has_proxy_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("proxy")
    })
}

// 生成方法参数列表 (a: Type, b: Type)
fn generate_params(method: &TraitItemMethod) -> proc_macro2::TokenStream {
    let params: Vec<_> = method.sig.inputs.iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                Some(pat_type.pat.to_token_stream())
            } else {
                None
            }
        })
        .collect();
    
    quote! { #(#params),* }
}

// 生成方法参数声明 (a: Type, b: Type)
fn generate_param_declarations(method: &TraitItemMethod) -> proc_macro2::TokenStream {
    let params: Vec<_> = method.sig.inputs.iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                Some(pat_type.to_token_stream())
            } else {
                None
            }
        })
        .collect();
    
    quote! { #(#params),* }
}

// 生成返回类型
fn generate_return_type(return_type: &ReturnType) -> proc_macro2::TokenStream {
    match return_type {
        ReturnType::Default => quote! {},
        ReturnType::Type(_, ty) => quote! { -> #ty },
    }
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn proxy_for(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 解析属性参数
    let proxy_for_attr = parse_macro_input!(attr as ProxyForAttr);
    let trait_path = &proxy_for_attr.trait_path;
    let trait_ident = trait_path.segments.last().unwrap().ident.clone();
    
    // 解析结构体定义
    let mut struct_item = parse_macro_input!(item as ItemStruct);
    let struct_ident = struct_item.ident.clone();
    let vis = struct_item.vis.clone();
    
    // 查找 trait 中的所有方法
    let trait_methods = match find_trait_methods(trait_path) {
        Ok(methods) => methods,
        Err(e) => return TokenStream::from(quote! { compile_error!(#e); }),
    };
    
    // 生成 trait 实现
    let trait_impl = {
        let method_impls = trait_methods.iter().map(|method| {
            let sig = &method.sig;
            let ident = &sig.ident;
            let params = generate_params(method);
            let param_declarations = generate_param_declarations(method);
            let return_type = generate_return_type(&sig.output);
            let has_proxy = has_proxy_attr(&method.attrs);
            
            if has_proxy {
                // 对于带 #[proxy] 属性的方法，调用自身实现
                quote! {
                    #sig {
                        self.#ident(#params)
                    }
                }
            } else {
                // 对于普通方法，转发给 inner
                quote! {
                    #sig {
                        self.inner.#ident(#params)
                    }
                }
            }
        });
        
        quote! {
            #vis impl #trait_path for #struct_ident {
                #(#method_impls)*
            }
        }
    };
    
    // 组合结构体定义和 trait 实现
    let expanded = quote! {
        #struct_item
        #trait_impl
    };
    
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn proxy(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 只是标记方法需要被代理，实际处理在 proxy_for 宏中
    let _ = parse_macro_input!(attr as ProxyAttr);
    item
}
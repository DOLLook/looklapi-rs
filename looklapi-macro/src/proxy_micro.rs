use syn::{ImplItemFn, Path, TraitItem, parse_quote};

/// 核心辅助函数：生成Trait中每个方法的转发逻辑（同步/异步通用）
pub fn generate_trait_methods(trait_path: &Path) -> Vec<ImplItemFn> {
    // 1. 解析Trait定义，获取所有方法（这里通过"虚拟Trait"占位，实际会关联用户传入的Trait）
    let trait_def: syn::ItemTrait = parse_quote!(
        trait DummyTrait: #trait_path {
            // 占位，实际会从用户指定的Trait中提取方法
        }
    );

    // 2. 遍历 trait 的所有项，筛选出函数定义并生成转发实现
    trait_def
        .items
        .into_iter()
        .filter_map(|item| {
            // 只处理 trait 中的函数定义
            let TraitItem::Fn(trait_fn) = item else {
                return None;
            };

            let syn::TraitItemFn { attrs, sig, .. } = trait_fn;

            // 获取方法名称和参数列表
            let method_ident = &sig.ident;

            // 提取方法参数（除了 &self 或 &mut self 外的所有参数）
            let args: Vec<_> = sig
                .inputs
                .iter()
                .filter_map(|arg| {
                    if let syn::FnArg::Typed(pat_type) = arg {
                        Some(&pat_type.pat)
                    } else {
                        None
                    }
                })
                .collect();

            // 判断是否为异步方法
            let is_async = sig.asyncness.is_some();

            // 构建调用内部对象的方法体
            let body = if is_async {
                parse_quote! {
                    {
                        self.inner.#method_ident(#(#args),*).await
                    }
                }
            } else {
                parse_quote! {
                    {
                        self.inner.#method_ident(#(#args),*)
                    }
                }
            };

            // 构造完整的实现方法
            Some(ImplItemFn {
                attrs,
                vis: syn::Visibility::Inherited,
                defaultness: None,
                sig,
                block: body,
            })
        })
        .collect()
}

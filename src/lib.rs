extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Block, Expr, File, Item, ItemFn, Local, Pat, Stmt};

#[proc_macro_attribute]
pub fn mvc_views(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as File);
    let mut new_items = vec![];

    for item in input.items {
        if let Item::Fn(func) = item {
            new_items.push(rewrite_fn(func));
        } else {
            new_items.push(item);
        }
    }

    let output = File { shebang: input.shebang, attrs: input.attrs, items: new_items };
    TokenStream::from(quote!(#output))
}

fn rewrite_fn(mut func: ItemFn) -> Item {
    let ident = func.sig.ident.clone();
    let has_replacements = block_has_local_named(&func.block, "replacements");

    // Does function already end with a value?
    let ends_with_value = func.block.stmts.last().map(|s| match s {
        Stmt::Expr(_) => true,
        Stmt::Semi(expr, _) => matches!(
            expr,
            Expr::Return(_) | Expr::Block(_) | Expr::Call(_)
        ),
        _ => false,
    }).unwrap_or(false);

    if !ends_with_value {
        let replacement_hashmap = if has_replacements {
            quote! { Some(replacements.clone()) }
        } else {
            quote! { None::<std::collections::HashMap<String, String>> }
        };
        let tail: Stmt = syn::parse_quote! {{
            use std::path::Path;
            let controller: String = {
                let p = Path::new(file!());
                let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
                if stem == "mod" {
                    p.parent()
                        .and_then(|pp| pp.file_name())
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string()
                } else { stem.to_string() }
            };
            let view_name = stringify!(#ident);
            let form_path = format!("./src/views/{}/{}.html", controller, view_name);
            let content = std::fs::read_to_string(&form_path).unwrap_or_default();
            println!("Replacements: {:?}", #replacement_hashmap);
            let content = crate::renderer::render(content, &host, #replacement_hashmap);
            return HttpResponse::Ok().content_type("text/html").body(content);
        }};
        func.block.stmts.push(tail);
    }

    Item::Fn(func)
}

fn block_has_local_named(block: &Block, name: &str) -> bool {
    for stmt in &block.stmts {
        if let Stmt::Local(Local { pat, .. }) = stmt {
            if let Pat::Ident(pat_ident) = pat {
                if pat_ident.ident == name {
                    return true;
                }
            }
        }
    }
    false
}


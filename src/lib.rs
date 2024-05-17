extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemMod, File, Item, Stmt};

#[proc_macro_attribute]
pub fn mvc_views(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as File);

    let mut new_items = vec![];

    for item in input.items {
        if let Item::Fn(mut func) = item {
            let name = func.sig.ident.to_string();

            // Inject a default params definition at the start of the function
            let default_replacements: Stmt = syn::parse_quote! {
                let replacements: Option<std::collections::HashMap<String, String>> = None;
                // let mut replacements = HashMap::new();
            };
            func.block.stmts.insert(0, default_replacements);

            // Check if the function already ends with a value
            let last_stmt = func.block.stmts.last();
            let ends_with_value = match last_stmt {
                Some(syn::Stmt::Expr(_)) => true,
                Some(syn::Stmt::Semi(expr, _)) => match *expr {
                    syn::Expr::Return(_) => true,
                    syn::Expr::Block(_) => true,
                    syn::Expr::Call(_) => true,
                    _ => false,
                },
                _ => false,
            };

            if !ends_with_value {
                let block: Stmt = syn::parse_quote! {
                    {
                        let form_path = format!("./src/views/emails/{}.html", #name);
                        let content = std::fs::read_to_string(&form_path).unwrap_or_default();
                        let content = crate::renderer::render(content, &host, replacements.into());
                        return HttpResponse::Ok()
                            .content_type("text/html")
                            .body(content);
                    }
                };
                func.block.stmts.push(block);
            }

            new_items.push(Item::Fn(func));
        } else {
            new_items.push(item);
        }
    }

    let output = File {
        shebang: input.shebang,
        attrs: input.attrs,
        items: new_items,
    };

    TokenStream::from(quote!(#output))
}

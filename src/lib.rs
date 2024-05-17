extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemMod, Stmt};

#[proc_macro_attribute]
pub fn mvc_views(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemMod);
    let mod_name = input.ident.to_string();

    let mut output = input.clone();
    let mut new_items = vec![];

    if let Some((brace, items)) = input.content {
        for item in items {
            if let syn::Item::Fn(mut func) = item {
                let name = func.sig.ident.to_string();

                // Inject a default params definition at the start of the function
                let default_params: Stmt = syn::parse_quote! {
                    let params: Option<HashMap<String, String>> = None;
                };
                func.block.stmts.insert(0, default_params);

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
                            let form_path = format!("./src/views/{}/{}.html", #mod_name, #name);
                            let content = std::fs::read_to_string(&form_path).unwrap_or_default();
                            let content = crate::renderer::render(content, &host, params.as_ref());
                            return HttpResponse::Ok()
                                .content_type("text/html")
                                .body(content);
                        }
                    };
                    func.block.stmts.push(block);
                }

                new_items.push(syn::Item::Fn(func));
            } else {
                new_items.push(item);
            }
        }

        output.content = Some((brace, new_items));
    }

    TokenStream::from(quote!(#output))
}

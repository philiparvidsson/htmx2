use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{
    parse::{self, Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    visit_mut::{self, VisitMut},
    Block, Expr, ExprLit, ExprReturn, FnArg, ItemFn, Lit, Pat, ReturnType, Signature, Token, Type,
};

struct MacroArgs {
    args: Vec<Expr>,
}

impl MacroArgs {
    fn get_path(&self) -> Option<Expr> {
        match self.args.get(0) {
            Some(expr) => Some(expr.clone()),
            _ => None,
        }
    }
}

impl Parse for MacroArgs {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let args = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;

        Ok(Self {
            args: args.into_iter().collect(),
        })
    }
}

#[derive(Default)]
struct MaybeWrapReturns {
    first_block_visit: bool,
}

impl MaybeWrapReturns {
    pub fn default() -> Self {
        Self {
            first_block_visit: true,
        }
    }
}

// TODO: Fix this piece of crap.
impl VisitMut for MaybeWrapReturns {
    fn visit_expr_mut(&mut self, i: &mut Expr) {
        if let Expr::Return(ExprReturn {
            expr: ref mut ret_expr,
            ..
        }) = i
        {
            if let Some(expr) = ret_expr {
                match **expr {
                    Expr::Call(_) => (),
                    _ => {
                        *ret_expr = Some(Box::new(parse_quote! {
                            Ok(htmx2::HtmxResponse::from(#ret_expr))
                        }));
                    }
                }
            };
        }

        visit_mut::visit_expr_mut(self, i);
    }

    fn visit_block_mut(&mut self, i: &mut Block) {
        if self.first_block_visit {
            let last_stmt = i.stmts.last_mut();
            if let Some(last_stmt) = last_stmt {
                *last_stmt = parse_quote! {
                    return Ok(htmx2::HtmxResponse::from(#last_stmt));
                };
            }

            self.first_block_visit = false;
        }

        visit_mut::visit_block_mut(self, i);
    }
}

fn macro_inner(path: &str, func: &ItemFn) -> TokenStream {
    let ItemFn {
        sig:
            Signature {
                ident,
                inputs,
                output,
                ..
            },
        mut block,
        ..
    } = func.clone();

    // Make sure that the return type is correct.
    if let ReturnType::Type(_, ty) = &output {
        if let Type::Path(type_path) = &**ty {
            if !type_path.path.is_ident("HtmxResult") {
                abort!(
                    Span::call_site(),
                    "incorrect return type";
                    help = "htmx functions must have return type `HtmxResult` (or no return type specified)"
                )
            }
        }
    } else {
        // No return type specified, which we're ok with - we'll adjust it anyway.
    }

    let func_args = inputs
        .iter()
        .filter_map(|input| {
            if let FnArg::Typed(pat) = input {
                if let Pat::Ident(ident) = &*pat.pat {
                    return Some(parse_quote! { args.#ident });
                }
            }

            None
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    MaybeWrapReturns::default().visit_block_mut(&mut block);

    let maud_support = if cfg!(feature = "maud") {
        quote! {
            impl maud::Render for #ident {
                fn render_to(&self, output: &mut String) {
                    output.push_str(#path);
                }
            }
        }
    } else {
        quote! {}
    };

    let tokens = quote! {
        #[allow(non_camel_case_types)]
        struct #ident;

        impl #ident {
            async fn #ident(htmx: &mut htmx2::Htmx, #inputs) -> htmx2::HtmxResult {
                #block
            }

            async fn call(htmx: htmx2::Htmx) -> (htmx2::Htmx, htmx2::HtmxResult) {
                #[derive(Default, serde::Deserialize)]
                #[serde(default)]
                struct Args {
                    #inputs
                }

                let args: Args = serde_urlencoded::from_bytes(&htmx.req.body_bytes())
                    .expect("Could not deserialize body into function arguments");

                let mut htmx_mut = htmx;
                let result = Self::#ident(&mut htmx_mut, #(#func_args),*).await;
                (htmx_mut, result)
            }
        }

        inventory::submit!(htmx2::HtmxFn {
            func: |htmx| Box::pin(#ident::call(htmx)),
            path: #path
        });

        #maud_support
    };

    tokens.into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn htmx(args: TokenStream, input: TokenStream) -> TokenStream {
    let macro_args = parse_macro_input!(args as MacroArgs);
    let func = parse_macro_input!(input as ItemFn);

    let func_name = func.sig.ident.to_string();
    let is_page = func_name.ends_with("_page");

    // The path at which the handler will be registered with the router. For
    // pages, we don't want to provide a default path, so we'll fail if one
    // hasn't been specified.
    let path = match macro_args.get_path() {
        Some(Expr::Lit(ExprLit {
            lit: Lit::Str(path),
            ..
        })) => path.value(),
        Some(_) => {
            abort!(
                Span::call_site(),
                "invalid path specified";
                help = "specify a path for the page: `#[htmx_page(\"/home\")]`"
            )
        }
        None if !is_page => format!("/api/{}", func.sig.ident),
        None => {
            abort!(
                Span::call_site(),
                "no path specified";
                help = "specify a path for the page: `#[htmx_page(\"/home\")]`"
            )
        }
    };

    macro_inner(&path, &func)
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn htmx_api(args: TokenStream, input: TokenStream) -> TokenStream {
    let macro_args = parse_macro_input!(args as MacroArgs);
    let func = parse_macro_input!(input as ItemFn);

    let path = match macro_args.get_path() {
        Some(Expr::Lit(ExprLit {
            lit: Lit::Str(path),
            ..
        })) => path.value(),
        _ => format!("/api/{}", func.sig.ident),
    };

    macro_inner(&path, &func)
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn htmx_page(args: TokenStream, input: TokenStream) -> TokenStream {
    let macro_args = parse_macro_input!(args as MacroArgs);
    let func = parse_macro_input!(input as ItemFn);

    let path = match macro_args.get_path() {
        Some(Expr::Lit(ExprLit {
            lit: Lit::Str(path),
            ..
        })) => path.value(),
        Some(_) => {
            abort!(
                Span::call_site(),
                "invalid path specified";
                help = "specify a path for the page: `#[htmx_page(\"/home\")]`"
            )
        }
        None => {
            abort!(
                Span::call_site(),
                "no path specified";
                help = "specify a path for the page: `#[htmx_page(\"/home\")]`"
            )
        }
    };

    macro_inner(&path, &func)
}

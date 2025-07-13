extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn register_command(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;

    let expanded = quote! {
        #input_fn

        inventory::submit! {
            crate::commands::CommandRegistration {
                constructor: || #fn_name(),
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn register_event(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let block = &input_fn.block;

    let wrapper_name = syn::Ident::new(
        &format!("__inventory_wrapper_{}", fn_name),
        fn_name.span(),
    );

    let expanded = quote! {
        #vis #sig #block

        fn #wrapper_name<'a>(
            ctx: &'a poise::serenity_prelude::Context,
            event: &'a poise::serenity_prelude::FullEvent,
            fw: poise::FrameworkContext<'a, crate::Data, crate::FullError>,
            data: &'a crate::Data,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), crate::FullError>> + Send + 'a>> {
            Box::pin(#fn_name(ctx, event, fw, data))
        }


        inventory::submit! {
            crate::events::EventHandlerRegistration {
                handler: #wrapper_name,
            }
        }
    };

    TokenStream::from(expanded)
}

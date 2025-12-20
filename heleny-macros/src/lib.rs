use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(BaseService)]
pub fn base_service(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let name_str = name.to_string();
    let expanded = quote! {
        use heleny_service::HasEndpoint;

        impl HasEndpoint for #name {
            fn endpoint(&mut self) ->  &mut heleny_bus::Endpoint {
                &mut self.endpoint
            }
        }

        use heleny_service::HasName;

        impl HasName for #name {
            fn name() -> &'static str {
                #name_str
            }
        }
    };

    TokenStream::from(expanded)
}

// #[proc_macro_derive(HasEndpoint)]
// pub fn has_endpoint(input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as DeriveInput);
//     let name = input.ident;

//     let expanded = quote! {
//         use heleny_service::HasEndpoint;

//         impl HasEndpoint for #name {
//             fn endpoint(&mut self) ->  &mut heleny_bus::Endpoint {
//                 &mut self.endpoint
//             }
//         }
//     };

//     TokenStream::from(expanded)
// }

// #[proc_macro_derive(HasName)]
// pub fn has_name(input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as DeriveInput);
//     let name = input.ident;
//     let name_str = name.to_string();
//     let expanded = quote! {
//         use heleny_service::HasName;

//         impl HasName for #name {
//             fn name() -> &'static str {
//                 #name_str
//             }
//         }
//     };

//     TokenStream::from(expanded)
// }
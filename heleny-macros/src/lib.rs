use proc_macro::TokenStream;
use quote::quote;
use syn::ItemStruct;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn base_service(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(input as ItemStruct);
    let name = &item_struct.ident;
    let name_str = name.to_string();

    let mut deps = Vec::new();

    // 使用更健壮的参数解析方式
    let args_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("deps") {
            let value = meta.value()?;
            let array: syn::ExprArray = value.parse()?;

            for expr in array.elems {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = expr
                {
                    deps.push(s.value());
                }
            }
        }
        Ok(())
    });

    parse_macro_input!(args with args_parser);

    let expanded = quote! {
        #item_struct

        use heleny_service::HasEndpoint;
        impl HasEndpoint for #name {
            fn endpoint_mut(&mut self) -> &mut heleny_bus::endpoint::Endpoint {
                &mut self.endpoint
            }
            fn endpoint(&self) -> &heleny_bus::endpoint::Endpoint {
                &self.endpoint
            }
        }

        use heleny_service::HasName;
        impl HasName for #name {
            fn name() -> &'static str {
                #name_str
            }
        }

        inventory::submit! {
            heleny_service::ServiceFactory {
                name: #name_str,
                deps: &[ #(#deps),* ],
                launch: |ep| {
                    #name::start(ep)
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn chat_model(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(input as ItemStruct);
    let name = &item_struct.ident;

    let expanded = quote! {
        #item_struct

        impl ChatModel for #name {
            fn schema(&self) -> &'static str {
                self.schema
            }
            fn client(&self) -> &Client<OpenAIConfig> {
                &self.client
            }
            fn model(&self) -> String {
                self.model.clone()
            }
            fn preset(&self) -> String {
                self.preset.clone()
            }
            fn timeout_secs(&self) -> u64 {
                self.timeout
            }
        }
    };

    TokenStream::from(expanded)
}

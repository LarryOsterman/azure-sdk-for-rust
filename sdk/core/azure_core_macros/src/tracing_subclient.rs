// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, ExprStruct, ItemFn, Result};
use tracing::error;

const INVALID_SUBCLIENT_MESSAGE: &str =
    "subclient attribute must be applied to a public function which returns a client type";

/// Parse the token stream for an Azure Service subclient declaration.
///
/// An Azure Service client is a public struct that represents a client for an Azure service.
///
///
pub fn parse_subclient(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    if !is_subclient_declaration(&item) {
        return Err(syn::Error::new(item.span(), INVALID_SUBCLIENT_MESSAGE));
    }

    let ItemFn {
        vis,
        sig,
        block,
        attrs,
    } = syn::parse2(item.clone())?;

    let body = block.stmts;

    let ExprStruct { fields, path, .. } = syn::parse2(body.first().unwrap().to_token_stream())?;

    let fields = fields.iter();

    Ok(quote! {
        #(#attrs)*
        #vis #sig {
            #path {
                #(#fields),*,
                tracer: self.tracer.clone(),
            }
        }
    })
}

fn is_subclient_declaration(item: &TokenStream) -> bool {
    let ItemFn {
        vis, block, sig, ..
    } = match syn::parse2(item.clone()) {
        Ok(fn_item) => fn_item,
        Err(e) => {
            error!("Failed to parse function: {}", e);
            return false;
        }
    };

    // Subclient constructors must be public functions.
    if !matches!(vis, syn::Visibility::Public(_)) {
        error!("Subclient constructors must be public functions");
        return false;
    }

    // Subclient constructors must have a body with a single statement.
    if block.stmts.len() != 1 {
        error!("Subclient constructors must have a single statement in their body");
        return false;
    }

    // Subclient constructors must have a return type that is a client type.
    if let syn::ReturnType::Type(_, ty) = &sig.output {
        if !matches!(ty.as_ref(), syn::Type::Path(p) if p.path.segments.last().unwrap().ident.to_string().ends_with("Client"))
        {
            return false;
        }
    } else {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracing::tests::setup_tracing;
    use proc_macro2::TokenStream;
    use quote::quote;
    use tracing::trace;

    #[test]
    fn test_is_subclient_declaration() {
        setup_tracing();
        assert!(is_subclient_declaration(&quote! {
            pub fn get_operation_templates_lro_client(&self) -> OperationTemplatesLroClient {
                OperationTemplatesLroClient {
                    api_version: self.api_version.clone(),
                    endpoint: self.endpoint.clone(),
                    pipeline: self.pipeline.clone(),
                    subscription_id: self.subscription_id.clone(),
                }
            }
        }),);

        assert!(!is_subclient_declaration(&quote! {
            pub fn not_a_subclient() {}
        }));

        assert!(is_subclient_declaration(&quote! {
            pub fn operation_templates_lro_client() -> OperationTemplatesLroClient {
                OperationTemplatesLroClient {
                    api_version: "2021-01-01".to_string(),
                    endpoint: "https://example.com".to_string(),
                    pipeline: "pipeline".to_string(),
                    subscription_id: "subscription_id".to_string(),
                }
            }
        }));
    }

    #[test]
    fn test_parse_subclient() {
        setup_tracing();
        let attr = TokenStream::new();
        let item = quote! {
            pub fn get_operation_templates_lro_client(&self) -> OperationTemplatesLroClient {
                OperationTemplatesLroClient {
                    api_version: self.api_version.clone(),
                    endpoint: self.endpoint.clone(),
                    pipeline: self.pipeline.clone(),
                    subscription_id: self.subscription_id.clone(),
                }
            }
        };

        let actual = parse_subclient(attr.clone(), item.clone())
            .expect("Failed to parse subclient declaration");
        trace!("Actual:{actual}");
        let expected = quote! {
            pub fn get_operation_templates_lro_client(&self) -> OperationTemplatesLroClient {
                OperationTemplatesLroClient {
                    api_version: self.api_version.clone(),
                    endpoint: self.endpoint.clone(),
                    pipeline: self.pipeline.clone(),
                    subscription_id: self.subscription_id.clone(),
                    tracer: self.tracer.clone(),
                }
            }
        };
        assert!(
            crate::tracing::tests::compare_token_stream(actual, expected),
            "Parsed tokens do not match expected tokens"
        );
    }
}

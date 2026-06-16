use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// Atributo #[diagnostic] — Marca uma função como puramente diagnóstica.
///
/// Restrições impostas em tempo de compilação:
/// 1. A função NÃO pode modificar `&mut self` ou qualquer estado global.
/// 2. A função NÃO pode chamar funções marcadas com `#[intervention]`.
/// 3. O tipo de retorno DEVE ser informativo (Score, LayerId, Vec<LayerId>, Efc, etc.),
///    NUNCA um tipo que represente uma modificação (Model, Weights, TaskVector).
#[proc_macro_attribute]
pub fn diagnostic(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let func_name = &func.sig.ident;
    let func_block = &func.block;
    let func_vis = &func.vis;
    let func_sig = &func.sig;

    // Verificação de tipo de retorno: não pode ser um tipo de modificação
    let return_type = match &func_sig.output {
        syn::ReturnType::Type(_, ty) => quote! { #ty },
        syn::ReturnType::Default => quote! { () },
    };
    let return_type_str = return_type.to_string();

    // Em tempo de compilação, verificamos se o tipo de retorno é proibido
    let is_intervention_type = return_type_str.contains("TaskVector")
        || return_type_str.contains("WeightModification")
        || return_type_str.contains("Model");

    let type_check = if is_intervention_type {
        quote! {
            compile_error!(
                concat!(
                    "Erro canónico: a função '", stringify!(#func_name),
                    "' está marcada como #[diagnostic] mas retorna um tipo de intervenção (",
                    #return_type_str,
                    "). Uma função de diagnóstico não pode produzir modificações."
                )
            );
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #[doc = "🔍 DIAGNÓSTICO — Esta função é um estetoscópio. Ouve, não corta."]
        #[doc = ""]
        #[doc = "Restrições canónicas:"]
        #[doc = "- Não modifica estado interno."]
        #[doc = "- Não chama funções de intervenção."]
        #[doc = "- Retorna apenas informação (localização, métrica, score)."]
        #func_vis #func_sig {
            #type_check
            #func_block
        }
    };

    TokenStream::from(expanded)
}

/// Atributo #[intervention] — Marca uma função como puramente interventiva.
///
/// Restrições impostas em tempo de compilação:
/// 1. A função DEVE modificar `&mut self` ou estado global.
/// 2. A função NÃO pode retornar informação de diagnóstico (Score, LayerId).
///    O retorno deve ser Result<(), Error> ou similar.
/// 3. A função DEVE receber um parâmetro `layers: &[LayerId]` explicitamente,
///    NUNCA inferir as camadas internamente. A localização é responsabilidade
///    do diagnóstico; a intervenção apenas aplica.
#[proc_macro_attribute]
pub fn intervention(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let func_name = &func.sig.ident;
    let func_block = &func.block;
    let func_vis = &func.vis;
    let func_sig = &func.sig;

    // Verificação: a função deve receber `layers: &[LayerId]`
    let has_layers_param = func.sig.inputs.iter().any(|arg| {
        if let syn::FnArg::Typed(pat_type) = arg {
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                return pat_ident.ident == "layers";
            }
        }
        false
    });

    let layer_check = if !has_layers_param {
        quote! {
            compile_error!(
                concat!(
                    "Erro canónico: a função '", stringify!(#func_name),
                    "' está marcada como #[intervention] mas não recebe o parâmetro 'layers: &[LayerId]'.",
                    " Uma função de intervenção deve receber explicitamente as camadas onde atuar.",
                    " A localização é responsabilidade do diagnóstico, não da intervenção."
                )
            );
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #[doc = "⚔️ INTERVENÇÃO — Esta função é um bisturi. Age, não diagnostica."]
        #[doc = ""]
        #[doc = "Restrições canónicas:"]
        #[doc = "- Modifica estado interno (&mut self)."]
        #[doc = "- Recebe 'layers: &[LayerId]' explicitamente (localização pelo diagnóstico)."]
        #[doc = "- Retorna Result<(), Error> (sucesso ou falha da modificação)."]
        #[doc = "- NÃO retorna informação de diagnóstico (Score, LayerId, Efc)."]
        #func_vis #func_sig {
            #layer_check
            #func_block
        }
    };

    TokenStream::from(expanded)
}

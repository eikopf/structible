use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, GenericArgument, PathArguments, Type};

/// Extracts doc comment strings from a list of attributes.
///
/// Returns a Vec of the doc strings (the content after `///`).
/// Returns an empty Vec if there are no doc comments.
pub fn extract_doc_comments(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .filter_map(|attr| {
            if let syn::Meta::NameValue(meta) = &attr.meta {
                if let syn::Expr::Lit(expr_lit) = &meta.value {
                    if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                        return Some(lit_str.value());
                    }
                }
            }
            None
        })
        .collect()
}

/// Formats method documentation with optional field documentation appended.
///
/// If `field_docs` is non-empty, appends them under a "## Field Documentation" subheading.
pub fn format_method_doc(auto_doc: &str, field_docs: &[String]) -> TokenStream {
    if field_docs.is_empty() {
        let doc = auto_doc.to_string();
        quote! { #[doc = #doc] }
    } else {
        let mut full_doc = auto_doc.to_string();
        full_doc.push_str("\n\n## Field Documentation");
        for line in field_docs {
            full_doc.push('\n');
            full_doc.push_str(line);
        }
        quote! { #[doc = #full_doc] }
    }
}

/// If `ty` is `Option<T>`, returns `Some(T)`. Otherwise returns `None`.
pub fn extract_option_inner(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };

    if type_path.qself.is_some() {
        return None;
    }

    let segments = &type_path.path.segments;
    if segments.len() != 1 {
        return None;
    }

    let segment = &segments[0];
    if segment.ident != "Option" {
        return None;
    }

    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };

    if args.args.len() != 1 {
        return None;
    }

    let GenericArgument::Type(inner) = &args.args[0] else {
        return None;
    };

    Some(inner)
}

/// Converts a snake_case identifier to PascalCase.
///
/// Handles raw identifiers (e.g., `r#type`) by stripping the `r#` prefix.
pub fn to_pascal_case(ident: &syn::Ident) -> syn::Ident {
    let s = ident.to_string();
    // Strip the raw identifier prefix if present
    let s = s.strip_prefix("r#").unwrap_or(&s);
    let pascal: String = s
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect();

    syn::Ident::new(&pascal, ident.span())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_doc_comments() {
        let attrs: Vec<Attribute> = vec![
            syn::parse_quote!(#[doc = " First line"]),
            syn::parse_quote!(#[doc = " Second line"]),
            syn::parse_quote!(#[some_other_attr]),
        ];
        let docs = extract_doc_comments(&attrs);
        assert_eq!(docs, vec![" First line", " Second line"]);
    }

    #[test]
    fn test_extract_doc_comments_empty() {
        let attrs: Vec<Attribute> = vec![syn::parse_quote!(#[some_attr])];
        let docs = extract_doc_comments(&attrs);
        assert!(docs.is_empty());
    }

    #[test]
    fn test_format_method_doc_no_field_docs() {
        let result = format_method_doc("Auto doc.", &[]);
        let expected = quote! { #[doc = "Auto doc."] };
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_format_method_doc_with_field_docs() {
        let field_docs = vec![
            " Field doc line 1".to_string(),
            " Field doc line 2".to_string(),
        ];
        let result = format_method_doc("Auto doc.", &field_docs);
        let expected_doc =
            "Auto doc.\n\n## Field Documentation\n Field doc line 1\n Field doc line 2";
        let expected = quote! { #[doc = #expected_doc] };
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_to_pascal_case() {
        let ident = syn::Ident::new("foo_bar_baz", proc_macro2::Span::call_site());
        let result = to_pascal_case(&ident);
        assert_eq!(result.to_string(), "FooBarBaz");
    }

    #[test]
    fn test_to_pascal_case_raw_identifier() {
        let ident = syn::Ident::new_raw("type", proc_macro2::Span::call_site());
        let result = to_pascal_case(&ident);
        assert_eq!(result.to_string(), "Type");
    }

    #[test]
    fn test_extract_option_inner() {
        let ty: Type = syn::parse2(quote! { Option<String> }).unwrap();
        let inner = extract_option_inner(&ty);
        assert!(inner.is_some());

        let ty: Type = syn::parse2(quote! { String }).unwrap();
        let inner = extract_option_inner(&ty);
        assert!(inner.is_none());
    }
}

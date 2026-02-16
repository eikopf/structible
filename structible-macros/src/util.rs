use syn::{GenericArgument, PathArguments, Type};

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
    use quote::quote;

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

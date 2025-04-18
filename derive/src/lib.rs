extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Attribute, Data, DeriveInput, Fields, Lit, Meta, NestedMeta, parse_macro_input};

/// Derives the FromConf and ToConf traits for struct types
///
/// This attribute allows a struct to be serialized to and deserialized from
/// configuration format using the confetti-rs library.
///
/// # Example
///
/// ```rust
/// use confetti_rs::ConfMap;
///
/// #[derive(ConfMap, Debug)]
/// struct ServerConfig {
///     port: i32,
///     host: String,
///     #[conf_map(name = "max-connections")]
///     max_connections: Option<i32>,
/// }
/// ```
///
/// # Attributes
///
/// - `#[conf_map(name = "field-name")]`: Specify a custom name for the field in the configuration
#[proc_macro_derive(ConfMap, attributes(conf_map))]
pub fn derive_conf_map(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let name_str = name.to_string();

    let (impl_from_conf, impl_to_conf) = match &input.data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields_named) => {
                    let from_conf_fields = fields_named.named.iter().map(|field| {
                        let field_name = field.ident.as_ref().unwrap();
                        let field_name_str = field_name.to_string();
                        let field_type = &field.ty;

                        // Check for conf_map attributes
                        let conf_name = get_conf_name_from_attrs(&field.attrs, &field_name_str);
                        let is_optional = is_option_type(field_type);

                        if is_optional {
                            quote! {
                                #field_name: {
                                    if let Some(child) = directive.children.iter().find(|d| d.name.value == #conf_name) {
                                        if !child.arguments.is_empty() {
                                            Some(confetti_rs::mapper::ValueConverter::from_conf_value(&child.arguments[0].value)?)
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                }
                            }
                        } else {
                            quote! {
                                #field_name: {
                                    if let Some(child) = directive.children.iter().find(|d| d.name.value == #conf_name) {
                                        if !child.arguments.is_empty() {
                                            confetti_rs::mapper::ValueConverter::from_conf_value(&child.arguments[0].value)?
                                        } else {
                                            return Err(confetti_rs::mapper::MapperError::MissingField(#conf_name.to_string()));
                                        }
                                    } else {
                                        return Err(confetti_rs::mapper::MapperError::MissingField(#conf_name.to_string()));
                                    }
                                }
                            }
                        }
                    });

                    let to_conf_fields = fields_named.named.iter().map(|field| {
                        let field_name = field.ident.as_ref().unwrap();
                        let field_name_str = field_name.to_string();

                        // Check for conf_map attributes
                        let conf_name = get_conf_name_from_attrs(&field.attrs, &field_name_str);
                        let is_optional = is_option_type(&field.ty);

                        if is_optional {
                            quote! {
                                if let Some(value) = &self.#field_name {
                                    let arg_value = confetti_rs::mapper::ValueConverter::to_conf_value(value)?;
                                    let arg = confetti_rs::ConfArgument {
                                        value: arg_value,
                                        span: 0..0,
                                        is_quoted: true,
                                        is_triple_quoted: false,
                                        is_expression: false,
                                    };

                                    let child = confetti_rs::ConfDirective {
                                        name: confetti_rs::ConfArgument {
                                            value: #conf_name.to_string(),
                                            span: 0..0,
                                            is_quoted: false,
                                            is_triple_quoted: false,
                                            is_expression: false,
                                        },
                                        arguments: vec![arg],
                                        children: vec![],
                                    };

                                    children.push(child);
                                }
                            }
                        } else {
                            quote! {
                                let arg_value = confetti_rs::mapper::ValueConverter::to_conf_value(&self.#field_name)?;
                                let arg = confetti_rs::ConfArgument {
                                    value: arg_value,
                                    span: 0..0,
                                    is_quoted: true,
                                    is_triple_quoted: false,
                                    is_expression: false,
                                };

                                let child = confetti_rs::ConfDirective {
                                    name: confetti_rs::ConfArgument {
                                        value: #conf_name.to_string(),
                                        span: 0..0,
                                        is_quoted: false,
                                        is_triple_quoted: false,
                                        is_expression: false,
                                    },
                                    arguments: vec![arg],
                                    children: vec![],
                                };

                                children.push(child);
                            }
                        }
                    });

                    let from_impl = quote! {
                        impl confetti_rs::FromConf for #name {
                            fn from_directive(directive: &confetti_rs::ConfDirective) -> Result<Self, confetti_rs::MapperError> {
                                if directive.name.value != #name_str {
                                    return Err(confetti_rs::MapperError::ParseError(
                                        format!("Expected directive name {}, found {}", #name_str, directive.name.value)
                                    ));
                                }

                                Ok(Self {
                                    #(#from_conf_fields),*
                                })
                            }
                        }
                    };

                    let to_impl = quote! {
                        impl confetti_rs::ToConf for #name {
                            fn to_directive(&self) -> Result<confetti_rs::ConfDirective, confetti_rs::MapperError> {
                                let mut children = Vec::new();

                                #(#to_conf_fields)*

                                Ok(confetti_rs::ConfDirective {
                                    name: confetti_rs::ConfArgument {
                                        value: #name_str.to_string(),
                                        span: 0..0,
                                        is_quoted: false,
                                        is_triple_quoted: false,
                                        is_expression: false,
                                    },
                                    arguments: vec![],
                                    children,
                                })
                            }
                        }
                    };

                    (from_impl, to_impl)
                }
                _ => {
                    // Only supports named fields
                    return syn::Error::new(
                        data_struct.fields.span(),
                        "ConfMap can only be derived for structs with named fields",
                    )
                    .to_compile_error()
                    .into();
                }
            }
        }
        _ => {
            // Only supports structs
            return syn::Error::new(input.span(), "ConfMap can only be derived for structs")
                .to_compile_error()
                .into();
        }
    };

    let expanded = quote! {
        #impl_from_conf

        #impl_to_conf
    };

    expanded.into()
}

// Helper functions

fn get_conf_name_from_attrs(attrs: &[Attribute], default_name: &str) -> String {
    for attr in attrs {
        if attr.path.is_ident("conf_map") {
            if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                for nested_meta in meta_list.nested.iter() {
                    if let NestedMeta::Meta(Meta::NameValue(name_value)) = nested_meta {
                        if name_value.path.is_ident("name") {
                            if let Lit::Str(lit_str) = &name_value.lit {
                                return lit_str.value();
                            }
                        }
                    }
                }
            }
        }
    }

    // Return the field name as default
    default_name.to_string()
}

fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

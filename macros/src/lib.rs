use djanho_core::metadata::{
    FieldMetadata,
    FieldTypeMetadata,
    StructMetadata,
};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// Registra uma struct como model do Djanho.
#[proc_macro_derive(Djanho)]
pub fn djanho_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;

    let extracted_fields = match &ast.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &ast,
                    "Djanho só suporta structs com campos nomeados",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                &ast,
                "Djanho só pode ser usado em structs",
            )
            .to_compile_error()
            .into();
        }
    };

    let mut fields = Vec::new();

    for field in extracted_fields {
        let Some(field_name) = field.ident.as_ref() else {
            continue;
        };

        fields.push(FieldMetadata::new(
            field_name.to_string(),
            field.ty.clone(),
        ));
    }

    let struct_metadata =
        StructMetadata::new(struct_name.to_string(), fields);

    let mut generated_fields = Vec::new();

    for field in &struct_metadata.fields {
        let Some(type_metadata) = field.type_metadata() else {
            return syn::Error::new_spanned(
                &field.target_type,
                format!(
                    "O tipo do campo '{}' não é suportado pelo Djanho",
                    field.name
                ),
            )
            .to_compile_error()
            .into();
        };

        let field_name = &field.name;
        let optional = type_metadata.optional;

        let kind = match type_metadata.field_type {
            FieldTypeMetadata::Text => {
                quote!(::djanho_core::model_entry::FieldKind::Text)
            }
            FieldTypeMetadata::Character => {
                quote!(::djanho_core::model_entry::FieldKind::Character)
            }
            FieldTypeMetadata::Integer => {
                quote!(::djanho_core::model_entry::FieldKind::Integer)
            }
            FieldTypeMetadata::Real => {
                quote!(::djanho_core::model_entry::FieldKind::Real)
            }
            FieldTypeMetadata::Boolean => {
                quote!(::djanho_core::model_entry::FieldKind::Boolean)
            }
            FieldTypeMetadata::Choice(choice_name) => {
                quote!(
                    ::djanho_core::model_entry::FieldKind::Choice(#choice_name)
                )
            }
        };

        generated_fields.push(quote! {
            ::djanho_core::model_entry::FieldEntry {
                name: #field_name,
                kind: #kind,
                optional: #optional,
            }
        });
    }

    let struct_name_str = struct_metadata.name.clone();

    quote! {
        ::djanho_core::inventory::submit! {
            ::djanho_core::model_entry::ModelEntry {
                name: #struct_name_str,
                fields: &[#(#generated_fields),*],
            }
        }
    }
    .into()
}

/// Registra um enum como conjunto de escolhas do Djanho.
///
/// Apenas variantes simples, sem campos associados, são aceitas:
/// `Male`, `Female`, `Other`.
#[proc_macro_derive(DjanhoChoice)]
pub fn djanho_choice_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let enum_name = &ast.ident;

    let variants = match &ast.data {
        Data::Enum(data) => &data.variants,
        _ => {
            return syn::Error::new_spanned(
                &ast,
                "DjanhoChoice só pode ser usado em enums",
            )
            .to_compile_error()
            .into();
        }
    };

    if variants.is_empty() {
        return syn::Error::new_spanned(
            &ast,
            "DjanhoChoice exige pelo menos uma variante",
        )
        .to_compile_error()
        .into();
    }

    let mut variant_names = Vec::new();

    for variant in variants {
        if !matches!(&variant.fields, Fields::Unit) {
            return syn::Error::new_spanned(
                variant,
                "DjanhoChoice suporta apenas variantes sem dados associados",
            )
            .to_compile_error()
            .into();
        }

        variant_names.push(variant.ident.to_string());
    }

    let enum_name_str = enum_name.to_string();

    quote! {
        ::djanho_core::inventory::submit! {
            ::djanho_core::choice_entry::ChoiceEntry {
                name: #enum_name_str,
                variants: &[#(#variant_names),*],
            }
        }
    }
    .into()
}

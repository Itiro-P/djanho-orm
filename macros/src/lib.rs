use djanho_core::metadata::{FieldMetadata, FieldTypeMetadata, StructMetadata};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Lê os atributos de relacionamento de um campo.
fn parse_relationships(field: &syn::Field) -> syn::Result<(Option<String>, Option<String>)> {
    let mut references = None;
    let mut many_to_many = None;

    for attr in &field.attrs {
        if !attr.path().is_ident("djanho") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("foreign_key") {
                if references.is_some() {
                    return Err(meta.error("foreign_key foi declarado mais de uma vez"));
                }

                let value: syn::LitStr = meta.value()?.parse()?;
                references = Some(value.value());
                Ok(())
            } else if meta.path.is_ident("many_to_many") {
                if many_to_many.is_some() {
                    return Err(meta.error("many_to_many foi declarado mais de uma vez"));
                }

                let value: syn::LitStr = meta.value()?.parse()?;
                many_to_many = Some(value.value());
                Ok(())
            } else {
                Err(meta.error("atributo desconhecido; use foreign_key ou many_to_many"))
            }
        })?;
    }

    if references.is_some() && many_to_many.is_some() {
        return Err(syn::Error::new_spanned(
            field,
            "um campo não pode ser foreign_key e many_to_many ao mesmo tempo",
        ));
    }

    Ok((references, many_to_many))
}

/// Registra uma struct como model do Djanho.
#[proc_macro_derive(Djanho, attributes(djanho))]
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
            return syn::Error::new_spanned(&ast, "Djanho só pode ser usado em structs")
                .to_compile_error()
                .into();
        }
    };

    let mut fields = Vec::new();
    let mut many_to_many_relations = Vec::new();

    for field in extracted_fields {
        let field_name = field.ident.as_ref().unwrap().to_string();

        if field_name == "id" {
            return syn::Error::new_spanned(
                field,
                "O campo 'id' é gerado automaticamente e não deve ser declarado",
            )
            .to_compile_error()
            .into();
        }

        let (references, many_to_many) = match parse_relationships(field) {
            Ok(result) => result,
            Err(error) => return error.to_compile_error().into(),
        };

        if let Some(target_model) = many_to_many {
            many_to_many_relations.push((field_name, target_model));

            continue;
        }

        fields.push(FieldMetadata::new(field_name, field.ty.clone(), references));
    }

    let struct_metadata = StructMetadata::new(struct_name.to_string(), fields);

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

        if field.references.is_some() && type_metadata.field_type != FieldTypeMetadata::Integer {
            return syn::Error::new_spanned(
                &field.target_type,
                format!(
                    "o campo '{}' é uma foreign_key e precisa ser um inteiro (i32, i64...)",
                    field.name
                ),
            )
            .to_compile_error()
            .into();
        }

        let references = match &field.references {
            Some(target) => quote!(::core::option::Option::Some(#target)),
            None => quote!(::core::option::Option::None),
        };

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
                references: #references,
            }
        });
    }

    let struct_name_str = struct_metadata.name.clone();

    let generated_relations = many_to_many_relations
        .iter()
        .map(|(field_name, target_model)| {
            quote! {
                ::djanho_core::inventory::submit! {
                    ::djanho_core::model_entry::ManyToManyEntry {
                        source_model: #struct_name_str,
                        field_name: #field_name,
                        target_model: #target_model,
                    }
                }
            }
        });

    quote! {
        ::djanho_core::inventory::submit! {
            ::djanho_core::model_entry::ModelEntry {
                name: #struct_name_str,
                fields: &[#(#generated_fields),*],
            }
        }

        #(#generated_relations)*
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
            return syn::Error::new_spanned(&ast, "DjanhoChoice só pode ser usado em enums")
                .to_compile_error()
                .into();
        }
    };

    if variants.is_empty() {
        return syn::Error::new_spanned(&ast, "DjanhoChoice exige pelo menos uma variante")
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

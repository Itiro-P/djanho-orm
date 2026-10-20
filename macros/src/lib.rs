//! Macros que leem structs e enums em tempo de compilação.
//!
//! `Djanho` e `DjanhoChoice` transformam as declarações do utilizador em
//! metadados estáticos que o `djanho-core` usa para gerar SQL e HTML.

use djanho_core::metadata::{FieldMetadata, FieldTypeMetadata, StructMetadata};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Confere se um campo marcado como N:N tem exatamente o formato `Vec<Modelo>`.
///
/// O atributo e o tipo são escritos separadamente pelo utilizador. Verificar
/// ambos aqui evita criar uma tabela intermediária apontando para um model
/// diferente daquele que o campo aparenta representar.
fn validate_many_to_many_type(ty: &syn::Type, expected: &str) -> syn::Result<()> {
    // Uma relação N:N precisa ser uma coleção de modelos, não uma coluna
    // individual. Atualmente, aceitamos a forma Vec<Modelo>.
    let syn::Type::Path(collection) = ty else {
        return Err(syn::Error::new_spanned(
            ty,
            "many_to_many exige um campo do tipo Vec<Model>",
        ));
    };

    let segment = collection.path.segments.last().ok_or_else(|| {
        syn::Error::new_spanned(ty, "many_to_many exige um campo do tipo Vec<Model>")
    })?;
    if segment.ident != "Vec" {
        return Err(syn::Error::new_spanned(
            ty,
            "many_to_many exige um campo do tipo Vec<Model>",
        ));
    }

    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return Err(syn::Error::new_spanned(
            ty,
            "many_to_many exige um campo do tipo Vec<Model>",
        ));
    };
    if arguments.args.len() != 1 {
        return Err(syn::Error::new_spanned(
            ty,
            "many_to_many exige um campo do tipo Vec<Model>",
        ));
    }

    let Some(syn::GenericArgument::Type(syn::Type::Path(target))) = arguments.args.first() else {
        return Err(syn::Error::new_spanned(
            ty,
            "many_to_many exige um campo do tipo Vec<Model>",
        ));
    };
    let target = target.path.segments.last().ok_or_else(|| {
        syn::Error::new_spanned(ty, "many_to_many exige um campo do tipo Vec<Model>")
    })?;
    if !matches!(target.arguments, syn::PathArguments::None) {
        return Err(syn::Error::new_spanned(
            ty,
            "many_to_many exige um campo do tipo Vec<Model>",
        ));
    }
    // Também conferimos que o tipo dentro do Vec é o mesmo modelo escrito no
    // atributo, evitando gerar uma associação com destino incorreto.
    if target.ident != expected {
        return Err(syn::Error::new_spanned(
            ty,
            format!(
                "o tipo do campo referencia '{}', mas many_to_many declara '{}'",
                target.ident, expected
            ),
        ));
    }

    Ok(())
}

/// Lê os atributos de relacionamento de um campo.
///
/// Exemplos aceitos:
/// - `#[djanho(foreign_key = "Author")]`
/// - `#[djanho(many_to_many = "Course")]`
/// - `#[djanho(many_to_many = "Person", relation = "Editors")]`
fn parse_relationships(
    field: &syn::Field,
) -> syn::Result<(Option<String>, Option<String>, Option<String>)> {
    let mut references = None;
    let mut many_to_many = None;
    let mut relation_name = None;

    for attr in &field.attrs {
        if !attr.path().is_ident("djanho") {
            continue;
        }

        // O parser permite agrupar atributos, como many_to_many e relation,
        // dentro de uma única anotação #[djanho(...)].
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
            } else if meta.path.is_ident("relation") {
                if relation_name.is_some() {
                    return Err(meta.error("relation foi declarado mais de uma vez"));
                }

                let value: syn::LitStr = meta.value()?.parse()?;
                relation_name = Some(value.value());
                Ok(())
            } else {
                Err(meta.error("atributo desconhecido; use foreign_key, many_to_many ou relation"))
            }
        })?;
    }

    if references.is_some() && many_to_many.is_some() {
        return Err(syn::Error::new_spanned(
            field,
            "um campo não pode ser foreign_key e many_to_many ao mesmo tempo",
        ));
    }

    if relation_name.is_some() && many_to_many.is_none() {
        return Err(syn::Error::new_spanned(
            field,
            "relation só pode ser usado junto com many_to_many",
        ));
    }

    Ok((references, many_to_many, relation_name))
}

/// Registra uma struct como model do Djanho.
#[proc_macro_derive(Djanho, attributes(djanho))]
pub fn djanho_derive(input: TokenStream) -> TokenStream {
    // syn converte o código-fonte em uma árvore que a macro consegue
    // inspecionar sem analisar texto manualmente.
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;

    // O gerador trabalha com structs de campos nomeados porque usa o nome de
    // cada campo como nome de coluna SQL e de controle HTML.
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

    // Primeira passagem: separar colunas comuns de relações N:N e validar
    // atributos associados a cada campo.
    for field in extracted_fields {
        // A coluna id é gerada pelo Djanho, então a struct não deve declará-la.
        let field_name = field.ident.as_ref().unwrap().to_string();

        if field_name == "id" {
            return syn::Error::new_spanned(
                field,
                "O campo 'id' é gerado automaticamente e não deve ser declarado",
            )
            .to_compile_error()
            .into();
        }

        let (references, many_to_many, relation_name) = match parse_relationships(field) {
            Ok(result) => result,
            Err(error) => return error.to_compile_error().into(),
        };

        if let Some(target_model) = many_to_many {
            if let Err(error) = validate_many_to_many_type(&field.ty, &target_model) {
                return error.to_compile_error().into();
            }

            many_to_many_relations.push((
                field_name,
                target_model,
                relation_name.unwrap_or_default(),
            ));

            continue;
        }

        // Campos comuns virarão colunas. N:N fica separado porque precisa de
        // uma tabela intermediária com duas chaves estrangeiras.
        fields.push(FieldMetadata::new(field_name, field.ty.clone(), references));
    }

    let struct_metadata = StructMetadata::new(struct_name.to_string(), fields);

    let mut generated_fields = Vec::new();

    // Segunda passagem: transformar os tipos reconhecidos em valores
    // FieldEntry que podem ser armazenados em memória estática.
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
                    "o campo '{}' é uma foreign_key e precisa ser um inteiro (i32)",
                    // "Colunas do tipo INTEGER PRIMARY KEY devem conter um inteiro com sinal de 32 bits.
                    // Qualquer tentativa de inserir dados que não sejam inteiros resultará em um erro."
                    // https://www-sqlite-org.translate.goog/datatypes.html?_x_tr_sl=en&_x_tr_tl=pt&_x_tr_hl=pt&_x_tr_pto=tc
                    field.name
                ),
            )
            .to_compile_error()
            .into();
        }

        // quote! prepara o código Rust que será produzido pela macro.
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

    // Cada relação é registrada em separado do model. O core irá reuni-las
    // depois e criar as tabelas intermediárias necessárias.
    let generated_relations =
        many_to_many_relations
            .iter()
            .map(|(field_name, target_model, relation_name)| {
                quote! {
                    // inventory permite reunir os models mesmo quando foram declarados
                    // em módulos diferentes do programa.
                    ::djanho_core::inventory::submit! {
                        ::djanho_core::model_entry::ManyToManyEntry {
                            source_model: #struct_name_str,
                            field_name: #field_name,
                            target_model: #target_model,
                            relation_name: #relation_name,
                        }
                    }
                }
            });

    quote! {
        // O registro do model contém somente colunas. As relações seguem em
        // registros próprios submetidos logo abaixo.
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

#[cfg(test)]
mod tests {
    use super::validate_many_to_many_type;
    use syn::parse_quote;

    // Estes testes verificam a validação do tipo sem precisar compilar um
    // programa separado para cada combinação de atributo.
    #[test]
    fn many_to_many_accepts_the_declared_vec_target() {
        let field_type = parse_quote!(Vec<crate::Course>);

        assert!(validate_many_to_many_type(&field_type, "Course").is_ok());
    }

    #[test]
    fn many_to_many_rejects_non_vec_fields() {
        let field_type = parse_quote!(Course);

        assert!(validate_many_to_many_type(&field_type, "Course").is_err());
    }

    #[test]
    fn many_to_many_rejects_mismatched_target_types() {
        let field_type = parse_quote!(Vec<Course>);

        assert!(validate_many_to_many_type(&field_type, "Student").is_err());
    }
}

/// Registra um enum como conjunto de escolhas do Djanho.
///
/// Apenas variantes simples, sem campos associados, são aceitas:
/// `Male`, `Female`, `Other`.
#[proc_macro_derive(DjanhoChoice)]
pub fn djanho_choice_derive(input: TokenStream) -> TokenStream {
    // Cada variante simples do enum será uma opção do formulário e um valor
    // aceito pela restrição SQL CHECK.
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
        // Variantes com dados não podem ser representadas por uma única opção
        // textual no formulário.
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

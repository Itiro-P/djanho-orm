use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Data, Fields};
use quote::quote;
use djanho_core::{metadata::{FieldMetadata, StructMetadata}};

/// Macro responsável por gerar o formulário de uma estrutura
#[proc_macro_derive(Djanho)]
pub fn djanho_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let struct_name = &ast.ident;

    let extracted_fields = match &ast.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Djanho só suporta structs com campos nomeados"),
        },
        _ => panic!("Djanho só suporta structs"),
    };

    let mut fields: Vec<FieldMetadata> = vec![];

    for field in extracted_fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = field.ty.clone();
        
        fields.push(FieldMetadata::new(field_name.to_string(), field_type));
    }

    let struct_metadata: StructMetadata = StructMetadata::new(struct_name.to_string(), fields);
    let struct_sql = struct_metadata.to_sql();
    let struct_html = struct_metadata.to_html();

    let struct_str = struct_name.to_string();
    let expanded = quote! {
        ::djanho_core::inventory::submit! {
            ::djanho_core::model_entry::ModelEntry {
                name: #struct_str,
                sql: #struct_sql,
                html: #struct_html,
            }
        }
    };

    return expanded.into();
    
}
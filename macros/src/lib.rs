use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro_derive(Djanho)]
pub fn djanho_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let nome_struct = &ast.ident;

    let campos = match &ast.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Djanho só suporta structs com campos nomeados"),
        },
        _ => panic!("Djanho só suporta structs"),
    };

    for campo in campos {
        let nome_campo = campo.ident.as_ref().unwrap();
        let tipo_campo = &campo.ty;
        let tipo_texto = quote!(#tipo_campo).to_string();
        println!("Field name: {} - Type: {}", nome_campo, tipo_texto);
    }

    TokenStream::new() // por enquanto
}
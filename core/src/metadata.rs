use syn::{GenericArgument, PathArguments, Type};

/// Tipo de campo descoberto durante a expansão do procedural macro.
///
/// Diferente de `model_entry::FieldKind`, esta versão pode possuir `String`,
/// porque existe apenas enquanto o macro está analisando o código-fonte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldTypeMetadata {
    Text,
    Character,
    Integer,
    Real,
    Boolean,
    Choice(String),
}

/// Resultado da análise de um `syn::Type`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeMetadata {
    pub field_type: FieldTypeMetadata,
    pub optional: bool,
}

/// Guarda as informações de um campo de uma estrutura durante a expansão do
/// macro.
pub struct FieldMetadata {
    pub name: String,
    pub target_type: Type,
    pub references: Option<String>,
}

impl FieldMetadata {
    pub fn new(name: String, target_type: Type, references: Option<String>) -> Self {
        Self {
            name,
            target_type,
            references,
        }
    }

    /// Interpreta o tipo Rust do campo para o conjunto de tipos suportados
    /// pelo Djanho.
    pub fn type_metadata(&self) -> Option<TypeMetadata> {
        analyze_type(&self.target_type)
    }

    pub fn print(&self) {
        let ty = &self.target_type;
        println!("Name -> {}; Type -> {}.", self.name, quote::quote!(#ty));
    }
}

/// Guarda as informações de uma estrutura durante a expansão do macro.
pub struct StructMetadata {
    pub name: String,
    pub fields: Vec<FieldMetadata>,
}

impl StructMetadata {
    pub fn new(name: String, fields: Vec<FieldMetadata>) -> Self {
        Self { name, fields }
    }

    pub fn print(&self) {
        println!("Struct: {}\nFields:", self.name);

        for field in &self.fields {
            print!(" - ");
            field.print();
        }
    }
}

/// Analisa um tipo Rust.
///
/// Exemplos:
/// - `String` -> `Text`, obrigatório
/// - `Option<String>` -> `Text`, opcional
/// - `i32` -> `Integer`, obrigatório
/// - `Gender` -> `Choice("Gender")`, obrigatório
pub fn analyze_type(ty: &Type) -> Option<TypeMetadata> {
    let (base_type, optional) = unwrap_option(ty);
    let field_type = classify_base_type(base_type)?;

    Some(TypeMetadata {
        field_type,
        optional,
    })
}

/// Remove uma ou mais camadas externas de `Option<T>`.
fn unwrap_option(mut ty: &Type) -> (&Type, bool) {
    let mut optional = false;

    while let Some(inner) = option_inner_type(ty) {
        optional = true;
        ty = inner;
    }

    (ty, optional)
}

/// Retorna o `T` de `Option<T>` quando `ty` for exatamente um `Option<T>`.
fn option_inner_type(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };

    let segment = type_path.path.segments.last()?;

    if segment.ident != "Option" {
        return None;
    }

    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };

    if arguments.args.len() != 1 {
        return None;
    }

    match arguments.args.first()? {
        GenericArgument::Type(inner) => Some(inner),
        _ => None,
    }
}

/// Classifica o tipo já sem `Option<T>`.
fn classify_base_type(ty: &Type) -> Option<FieldTypeMetadata> {
    let Type::Path(type_path) = ty else {
        return None;
    };

    let segment = type_path.path.segments.last()?;

    // Tipos genéricos desconhecidos, como Vec<String>, não são tratados como
    // enums por acidente.
    if !matches!(segment.arguments, PathArguments::None) {
        return None;
    }

    let name = segment.ident.to_string();

    let kind = match name.as_str() {
        "String" => FieldTypeMetadata::Text,
        "char" => FieldTypeMetadata::Character,

        "i8" | "i16" | "i32" | "i64" | "isize" | "u8" | "u16" | "u32" | "u64" | "usize" => {
            FieldTypeMetadata::Integer
        }

        "f32" | "f64" => FieldTypeMetadata::Real,
        "bool" => FieldTypeMetadata::Boolean,

        // Qualquer outro path simples é tratado como um Choice. Durante a
        // geração final ele precisa existir no inventory como ChoiceEntry.
        _ => FieldTypeMetadata::Choice(name),
    };

    Some(kind)
}

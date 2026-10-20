//! Analisa os tipos Rust encontrados nas structs para descobrir como
//! representá-los no SQL e nos formulários.

use syn::{GenericArgument, PathArguments, Type};

/// Tipo de campo descoberto durante a expansão do procedural macro.
///
/// Diferente de `model_entry::FieldKind`, esta versão pode possuir `String`,
/// porque existe apenas enquanto o macro está analisando o código-fonte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldTypeMetadata {
    /// Texto armazenado como `TEXT`.
    Text,
    /// Um único caractere, armazenado como texto.
    Character,
    /// Número inteiro, armazenado como `INTEGER`.
    Integer,
    /// Número com casas decimais, armazenado como `REAL`.
    Real,
    /// Valor verdadeiro/falso, armazenado como `BOOLEAN`.
    Boolean,
    /// Nome de um enum que precisa estar registrado com `DjanhoChoice`.
    Choice(String),
}

/// Resultado da análise de um `syn::Type`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeMetadata {
    /// Tipo básico do campo depois de remover `Option`, se houver.
    pub field_type: FieldTypeMetadata,
    /// Informa se o tipo Rust original permitia ausência (`Option<T>`).
    pub optional: bool,
}

/// Guarda as informações de um campo de uma estrutura durante a expansão do
/// macro.
pub struct FieldMetadata {
    /// Nome do campo na struct.
    pub name: String,
    /// Tipo original reconhecido pelo parser `syn`.
    pub target_type: Type,
    /// Model referenciado quando este campo é uma chave estrangeira.
    pub references: Option<String>,
}

impl FieldMetadata {
    /// Cria os dados temporários de um campo enquanto a macro o analisa.
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
        // Delega a análise para a mesma função usada pelos outros caminhos,
        // mantendo uma única regra para interpretar tipos Rust.
        analyze_type(&self.target_type)
    }

    /// Exibe o nome e o tipo do campo durante depuração manual.
    ///
    /// Não é usado durante a geração normal; é um auxílio quando se quer
    /// entender o que a macro encontrou no código-fonte.
    pub fn print(&self) {
        let ty = &self.target_type;
        println!("Nome -> {}; Tipo -> {}.", self.name, quote::quote!(#ty));
    }
}

/// Guarda as informações de uma estrutura durante a expansão do macro.
pub struct StructMetadata {
    /// Nome da struct que será registrada como model.
    pub name: String,
    /// Campos comuns da struct; relações N:N são armazenadas separadamente.
    pub fields: Vec<FieldMetadata>,
}

impl StructMetadata {
    /// Cria os dados temporários de uma struct que será registrada como model.
    pub fn new(name: String, fields: Vec<FieldMetadata>) -> Self {
        Self { name, fields }
    }

    /// Exibe a struct e seus campos durante depuração manual.
    pub fn print(&self) {
        println!("Struct: {}\nCampos:", self.name);

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
    // Primeiro descobrimos se é opcional; depois classificamos o tipo interno.
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

    // Cada camada de Option representa possibilidade de ausência. Rust
    // normalmente usa uma só, mas o laço evita deixar uma camada escondida.
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

    // Somente Option<T> altera a nulabilidade da coluna.
    if segment.ident != "Option" {
        return None;
    }

    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };

    // Option precisa ter exatamente um argumento de tipo: Option<String>.
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

    // Os tipos conhecidos têm uma tradução explícita. Um nome simples que
    // não aparece aqui é tratado como enum e validado depois contra o registro.
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

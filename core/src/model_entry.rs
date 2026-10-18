/// Tipo lógico de um campo conhecido pelo Djanho em tempo de execução.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Text,
    Character,
    Integer,
    Real,
    Boolean,
    /// Campo cujo tipo é um enum registrado com `DjanhoChoice`.
    Choice(&'static str),
}

/// Metadados de um campo de uma struct registrada com `#[derive(Djanho)]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldEntry {
    /// Nome do campo Rust. Também será usado como nome da coluna SQL e como
    /// atributo `name` do controle HTML.
    pub name: &'static str,
    pub kind: FieldKind,
    /// `true` quando o tipo original era `Option<T>`.
    pub optional: bool,
}

/// Metadados de uma struct registrada com `#[derive(Djanho)]`.
pub struct ModelEntry {
    pub name: &'static str,
    pub fields: &'static [FieldEntry],
}

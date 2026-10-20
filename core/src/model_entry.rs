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
    /// Nome do model apontado, quando o campo é uma chave estrangeira.
    pub references: Option<&'static str>,
}

/// Metadados de uma struct registrada com `#[derive(Djanho)]`.
pub struct ModelEntry {
    pub name: &'static str,
    pub fields: &'static [FieldEntry],
}

/// Metadados de um relacionamento muitos-para-muitos entre dois models.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManyToManyEntry {
    /// Model que declara o relacionamento.
    pub source_model: &'static str,

    /// Nome do campo que representa o relacionamento.
    pub field_name: &'static str,

    /// Model relacionado.
    pub target_model: &'static str,

    /// Identificador opcional para distinguir relações entre o mesmo par.
    /// String vazia indica a relação padrão.
    pub relation_name: &'static str,
}

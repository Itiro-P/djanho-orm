//! Estruturas que guardam a descrição dos models e relacionamentos depois
//! que as macros os registram.

/// Tipo lógico de um campo conhecido pelo Djanho em tempo de execução.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    /// Texto comum. No SQLite, será armazenado em uma coluna `TEXT`.
    Text,
    /// Um caractere. O gerador adiciona uma validação para aceitar só um caractere.
    Character,
    /// Número inteiro, incluindo os tipos inteiros com e sem sinal de Rust.
    Integer,
    /// Número decimal, convertido para uma coluna `REAL`.
    Real,
    /// Valor verdadeiro ou falso, armazenado como `0` ou `1`.
    Boolean,
    /// Campo cujo tipo é um enum registrado com `DjanhoChoice`.
    /// O nome guardado aqui permite encontrar as variantes registradas.
    Choice(&'static str),
}

/// Metadados de um campo de uma struct registrada com `#[derive(Djanho)]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldEntry {
    /// Nome do campo Rust. Também será usado como nome da coluna SQL e como
    /// atributo `name` do controle HTML.
    pub name: &'static str,
    /// Tipo do campo, usado para escolher tanto o tipo SQL quanto o controle HTML.
    pub kind: FieldKind,
    /// `true` quando o tipo original era `Option<T>`. No SQL, isso remove
    /// `NOT NULL`; no formulário, torna o campo opcional.
    pub optional: bool,
    /// Nome do model apontado quando o campo é uma chave estrangeira.
    /// O campo continua sendo inteiro, mas o banco verifica se o id existe.
    pub references: Option<&'static str>,
}

/// Metadados de uma struct registrada com `#[derive(Djanho)]`.
pub struct ModelEntry {
    /// Nome usado para a tabela SQL e para a rota HTTP, como `/Book`.
    pub name: &'static str,
    /// Campos escalares da struct. Relações N:N não são colunas e ficam em
    /// `ManyToManyEntry`, pois precisam de uma tabela intermediária.
    pub fields: &'static [FieldEntry],
}

/// Metadados de uma declaração de relacionamento muitos-para-muitos.
///
/// Cada campo anotado gera um registro. Se a mesma relação for declarada nos
/// dois models, o gerador compara os nomes e cria apenas uma tabela de ligação.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManyToManyEntry {
    /// Model onde o campo `Vec<...>` foi escrito.
    pub source_model: &'static str,

    /// Nome do campo `Vec<...>`, útil para identificar a declaração de origem.
    pub field_name: &'static str,

    /// Model contido no `Vec` e declarado também em `many_to_many = "..."`.
    pub target_model: &'static str,

    /// Identificador opcional para distinguir relações diferentes entre o
    /// mesmo par de models. A string vazia indica a relação padrão.
    pub relation_name: &'static str,
}

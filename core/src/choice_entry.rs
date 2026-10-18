/// Metadados de um enum registrado com `#[derive(DjanhoChoice)]`.
///
/// Cada variante do enum vira uma opção de um `<select>` no HTML e pode
/// também ser usada para restringir os valores aceitos no SQL.
pub struct ChoiceEntry {
    /// Nome do enum Rust.
    pub name: &'static str,
    /// Nomes das variantes do enum.
    pub variants: &'static [&'static str],
}

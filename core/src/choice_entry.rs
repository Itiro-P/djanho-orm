/// Metadados de um enum registrado com `#[derive(DjanhoChoice)]`.
///
/// Cada variante do enum vira uma opção de um `<select>` no HTML e pode
/// também ser usada para restringir os valores aceitos no SQL. Por exemplo,
/// `Gender::Male` vira a opção `"Male"` e o mesmo texto entra no `CHECK` SQL.
pub struct ChoiceEntry {
    /// Nome do enum Rust, usado para localizar este registro ao analisar um campo.
    pub name: &'static str,
    /// Nomes das variantes, na ordem em que aparecem no enum.
    pub variants: &'static [&'static str],
}

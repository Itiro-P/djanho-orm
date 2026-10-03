/// Estrutura usada para guardar os metadados de uma estrutura durante a derivação do macro.
/// 
/// Essa estrutura é consumida pelo Djanho internamente.
pub struct ModelEntry {
    pub name: &'static str,
    pub sql: &'static str,
    pub html: &'static str,
}
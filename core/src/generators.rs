use crate::model_entry::ModelEntry;

/// Gera o HTML unificado para todas as entradas fornecidas.
pub fn generate_form(entries: &Vec<&ModelEntry>) -> String {
    entries.iter()
        .filter(|e| !e.html.is_empty()).map(|e| format!("<!-- {} -->\n{}", e.name, e.html))
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Gera o SQL unificado para todas as entradas fornecidas.
pub fn generate_schema(entries: &Vec<&ModelEntry>) -> String {
    entries.iter()
        .filter(|e| !e.sql.is_empty()).map(|e| format!("-- {}\n{}\n", e.name, e.sql))
        .collect::<Vec<_>>()
        .join("\n\n")
}

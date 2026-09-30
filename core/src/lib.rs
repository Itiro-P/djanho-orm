pub mod metadata;
pub mod model_entry;
pub mod generators;

use std::fs::File;
use std::io::Write;

#[doc(hidden)]
pub use inventory;

inventory::collect!(model_entry::ModelEntry);

/// Junta o SQL e o HTML de todas as structs com #[derive(Djanho)] 
/// e cria um schema.sql e um form.html unificados.
pub fn generate() -> std::io::Result<()> {
    let mut entries: Vec<&model_entry::ModelEntry> = inventory::iter::<model_entry::ModelEntry>().into_iter().collect();
    entries.sort_by_key(|e| e.name);

    let html_content = generators::generate_form(&entries);
    let sql_content  = generators::generate_schema(&entries);

    File::create("form.html")?
        .write_all(html_content.as_bytes())?;

    File::create("schema.sql")?
        .write_all(sql_content.as_bytes())?;

    Ok(())
}

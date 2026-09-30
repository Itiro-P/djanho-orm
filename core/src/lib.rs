pub mod metadata;
pub mod model_entry;
mod generators;

#[doc(hidden)]
pub use inventory;

inventory::collect!(model_entry::ModelEntry);

/// Junta o SQL e o HTML de todas as structs com #[derive(Djanho)] e cria um schema.sql e um form.html unificado.
pub fn generate() -> std::io::Result<()> {
    let mut entries: Vec<&model_entry::ModelEntry> = inventory::iter::<model_entry::ModelEntry>.into_iter().collect();
    entries.sort_by_key(|e| e.name);

    for entry in entries {
        generate_files(entry)?;
    }

    return Ok(());
}

fn generate_files(entry: &model_entry::ModelEntry) -> std::io::Result<()> {
    generators::generate_schema(entry)?;
    generators::generate_form(entry)?;

    return Ok(());
}
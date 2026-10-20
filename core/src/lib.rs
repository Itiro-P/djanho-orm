pub mod choice_entry;
pub mod generators;
pub mod metadata;
pub mod model_entry;

use std::fs::File;
use std::io::{self, Write};

#[doc(hidden)]
pub use inventory;

inventory::collect!(model_entry::ModelEntry);
inventory::collect!(choice_entry::ChoiceEntry);
inventory::collect!(model_entry::ManyToManyEntry);

/// Junta os metadados registrados por `Djanho` e `DjanhoChoice` e cria
/// `docker/schema.sql` e `docker/form.html`.
pub fn generate() -> io::Result<()> {
    let mut entries: Vec<&model_entry::ModelEntry> = inventory::iter::<model_entry::ModelEntry>
        .into_iter()
        .collect();

    let mut choices: Vec<&choice_entry::ChoiceEntry> = inventory::iter::<choice_entry::ChoiceEntry>
        .into_iter()
        .collect();

    entries.sort_by_key(|entry| entry.name);
    choices.sort_by_key(|choice| choice.name);

    let mut registered_relations: Vec<&model_entry::ManyToManyEntry> =
        inventory::iter::<model_entry::ManyToManyEntry>
            .into_iter()
            .collect();

    registered_relations.sort_by_key(|relation| {
        (
            relation.source_model,
            relation.target_model,
            relation.field_name,
        )
    });

    let relations = generators::resolve_many_to_many_relations(&registered_relations)
        .map_err(|message| io::Error::new(io::ErrorKind::InvalidData, message))?;

    let html_content = generators::generate_form_with_relations(&entries, &choices, &relations)
        .map_err(|message| io::Error::new(io::ErrorKind::InvalidData, message))?;

    let sql_content = generators::generate_schema(&entries, &choices, &relations)
        .map_err(|message| io::Error::new(io::ErrorKind::InvalidData, message))?;

    // A pasta `docker/` já faz parte da estrutura do projeto.
    File::create("docker/form.html")?.write_all(html_content.as_bytes())?;

    File::create("docker/schema.sql")?.write_all(sql_content.as_bytes())?;

    Ok(())
}

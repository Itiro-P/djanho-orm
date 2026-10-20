//! Núcleo da biblioteca: mantém os metadados registrados e coordena a geração
//! dos arquivos SQL e HTML.

pub mod choice_entry;
pub mod generators;
pub mod metadata;
pub mod model_entry;

use std::fs::File;
use std::io::{self, Write};

#[doc(hidden)]
// As macros usam inventory para registrar descrições estáticas sem que o
// programa precise montar manualmente uma lista de todos os models.
pub use inventory;

// Cada tipo é uma categoria independente de dados que pode ser registrada.
inventory::collect!(model_entry::ModelEntry);
inventory::collect!(choice_entry::ChoiceEntry);
inventory::collect!(model_entry::ManyToManyEntry);

/// Junta os metadados registrados por `Djanho` e `DjanhoChoice` e cria
/// `docker/schema.sql` e `docker/form.html`.
pub fn generate() -> io::Result<()> {
    // inventory recolhe os itens submetidos pelas macros em qualquer módulo
    // que tenha sido carregado no programa.
    let mut entries: Vec<&model_entry::ModelEntry> = inventory::iter::<model_entry::ModelEntry>
        .into_iter()
        .collect();

    let mut choices: Vec<&choice_entry::ChoiceEntry> = inventory::iter::<choice_entry::ChoiceEntry>
        .into_iter()
        .collect();

    // Uma ordenação fixa facilita comparar os arquivos gerados no Git e
    // evita diferenças causadas pela ordem em que os módulos foram carregados.
    entries.sort_by_key(|entry| entry.name);
    choices.sort_by_key(|choice| choice.name);

    // Ordenar também as declarações de relação torna determinística a criação
    // das tabelas intermediárias e facilita comparar o schema gerado.
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

    // Geramos primeiro o HTML e o SQL em memória. Assim, erros de enum,
    // relacionamento ou referência são retornados antes de substituir arquivos.
    let html_content = generators::generate_form_with_relations(&entries, &choices, &relations)
        .map_err(|message| io::Error::new(io::ErrorKind::InvalidData, message))?;

    let sql_content = generators::generate_schema(&entries, &choices, &relations)
        .map_err(|message| io::Error::new(io::ErrorKind::InvalidData, message))?;

    // O exemplo Docker espera estes nomes e caminhos fixos.
    File::create("docker/form.html")?.write_all(html_content.as_bytes())?;

    File::create("docker/schema.sql")?.write_all(sql_content.as_bytes())?;

    Ok(())
}

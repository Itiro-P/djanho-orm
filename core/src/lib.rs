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

    // // Relacionamentos N:N configurados manualmente.
    // // Adicione relações aqui quando os dois modelos existirem.
    // let relations: Vec<generators::ManyToManyRelation> = vec![
    //     // Exemplo para habilitar quando os modelos estiverem registrados:
    //     //
    //     // generators::ManyToManyRelation {
    //     //     name: "Aluno_Disciplina".to_string(),
    //     //     left_model: "Aluno".to_string(),
    //     //     left_field: "aluno_id".to_string(),
    //     //     right_model: "Disciplina".to_string(),
    //     //     right_field: "disciplina_id".to_string(),
    //     // },
    // ];

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

    let mut relations: Vec<generators::ManyToManyRelation> = Vec::new();
    let mut processed_pairs = std::collections::BTreeSet::new();

    for relation in registered_relations {
        // Ordenar os modelos evita gerar duas tabelas quando a relação
        // também estiver declarada no modelo inverso.
        let (left_model, right_model) = if relation.source_model <= relation.target_model {
            (relation.source_model, relation.target_model)
        } else {
            (relation.target_model, relation.source_model)
        };

        if left_model == right_model {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Relacionamento N:N inválido: '{}' aponta para si mesmo",
                    left_model
                ),
            ));
        }

        if !processed_pairs.insert((left_model, right_model)) {
            continue;
        }

        relations.push(generators::ManyToManyRelation {
            name: format!("{left_model}_{right_model}_Link"),
            left_model: left_model.to_string(),
            left_field: format!("{}_id", left_model.to_lowercase()),
            right_model: right_model.to_string(),
            right_field: format!("{}_id", right_model.to_lowercase()),
        });
    }

    let html_content = generators::generate_form(&entries, &choices)
        .map_err(|message| io::Error::new(io::ErrorKind::InvalidData, message))?;

    let sql_content = generators::generate_schema(&entries, &choices, &relations)
        .map_err(|message| io::Error::new(io::ErrorKind::InvalidData, message))?;

    // A pasta `docker/` já faz parte da estrutura do projeto.
    File::create("docker/form.html")?.write_all(html_content.as_bytes())?;

    File::create("docker/schema.sql")?.write_all(sql_content.as_bytes())?;

    Ok(())
}

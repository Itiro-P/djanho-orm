use crate::choice_entry::ChoiceEntry;
use crate::model_entry::{FieldEntry, FieldKind, ManyToManyEntry, ModelEntry};

/// Representa um relacionamento muitos-para-muitos (N:N).
///
/// Exemplo: vários alunos podem cursar várias disciplinas.
#[derive(Debug, Clone)]
pub struct ManyToManyRelation {
    /// Nome da tabela intermediária.
    pub name: String,

    /// Primeiro modelo do relacionamento.
    pub left_model: String,

    /// Nome da coluna que referencia o primeiro modelo.
    pub left_field: String,

    /// Segundo modelo do relacionamento.
    pub right_model: String,

    /// Nome da coluna que referencia o segundo modelo.
    pub right_field: String,

    /// Identificador que distingue relações entre o mesmo par de models.
    pub relation_name: String,
}

/// Converte declarações N:N registradas nos models em tabelas intermediárias.
///
/// Uma relação declarada nos dois lados gera apenas uma tabela.
pub fn resolve_many_to_many_relations(
    entries: &[&ManyToManyEntry],
) -> Result<Vec<ManyToManyRelation>, String> {
    let mut relations = Vec::new();
    let mut processed_relations = std::collections::BTreeSet::new();

    for entry in entries {
        let (left_model, right_model) = if entry.source_model <= entry.target_model {
            (entry.source_model, entry.target_model)
        } else {
            (entry.target_model, entry.source_model)
        };

        if !processed_relations.insert((left_model, right_model, entry.relation_name)) {
            continue;
        }

        let self_relation = left_model == right_model;
        let relation_name = entry.relation_name;
        let name = if relation_name.is_empty() {
            format!("{left_model}_{right_model}_Link")
        } else {
            format!("{left_model}_{right_model}_{relation_name}_Link")
        };

        relations.push(ManyToManyRelation {
            name,
            left_model: left_model.to_string(),
            left_field: if self_relation {
                "source_id".to_string()
            } else {
                format!("{}_id", left_model.to_lowercase())
            },
            right_model: right_model.to_string(),
            right_field: if self_relation {
                "target_id".to_string()
            } else {
                format!("{}_id", right_model.to_lowercase())
            },
            relation_name: relation_name.to_string(),
        });
    }

    Ok(relations)
}

/// Gera o HTML unificado para todos os modelos registrados.
pub fn generate_form(entries: &[&ModelEntry], choices: &[&ChoiceEntry]) -> Result<String, String> {
    generate_form_with_relations(entries, choices, &[])
}

/// Gera formulários com controles para foreign keys e relacionamentos N:N.
pub fn generate_form_with_relations(
    entries: &[&ModelEntry],
    choices: &[&ChoiceEntry],
    relations: &[ManyToManyRelation],
) -> Result<String, String> {
    let mut forms = Vec::new();

    for entry in entries {
        let has_many_to_many = relations.iter().any(|relation| {
            relation.left_model == entry.name || relation.right_model == entry.name
        });
        if entry.fields.is_empty() && !has_many_to_many {
            continue;
        }

        forms.push(generate_model_form(entry, choices, relations)?);
    }

    forms.push(generate_css());

    Ok(forms.join("\n\n"))
}

/// Gera o SQL unificado para todos os modelos e
/// relacionamentos muitos-para-muitos registrados.
pub fn generate_schema(
    entries: &[&ModelEntry],
    choices: &[&ChoiceEntry],
    relations: &[ManyToManyRelation],
) -> Result<String, String> {
    validate_references(entries)?;
    validate_many_to_many_relations(relations, entries)?;

    let mut schemas = Vec::new();

    // Gera primeiro as tabelas dos modelos.
    for entry in order_by_dependency(entries) {
        let is_relationship_model = relations.iter().any(|relation| {
            relation.left_model == entry.name || relation.right_model == entry.name
        });
        if entry.fields.is_empty() && !is_relationship_model {
            continue;
        }

        schemas.push(generate_model_schema(entry, choices)?);
    }

    // Gera as tabelas intermediárias dos relacionamentos N:N.
    for relation in relations {
        schemas.push(generate_many_to_many_schema(relation, entries)?);
    }

    Ok(schemas.join("\n\n"))
}

fn validate_many_to_many_relations(
    relations: &[ManyToManyRelation],
    entries: &[&ModelEntry],
) -> Result<(), String> {
    let mut table_names = std::collections::HashSet::new();

    for relation in relations {
        for (kind, identifier) in [
            ("tabela", relation.name.as_str()),
            ("model", relation.left_model.as_str()),
            ("coluna", relation.left_field.as_str()),
            ("model", relation.right_model.as_str()),
            ("coluna", relation.right_field.as_str()),
        ] {
            if !is_sql_identifier(identifier) {
                return Err(format!(
                    "O nome da {kind} '{}' do relacionamento N:N é inválido",
                    identifier
                ));
            }
        }

        if entries.iter().any(|entry| entry.name == relation.name) {
            return Err(format!(
                "A tabela intermediária '{}' conflita com o nome de um model",
                relation.name
            ));
        }
        if !table_names.insert(relation.name.as_str()) {
            return Err(format!(
                "Mais de um relacionamento N:N usa a tabela '{}'",
                relation.name
            ));
        }
    }

    Ok(())
}

fn is_sql_identifier(identifier: &str) -> bool {
    let mut characters = identifier.chars();
    matches!(characters.next(), Some(first) if first == '_' || first.is_ascii_alphabetic())
        && characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

/// Garante que toda foreign key aponta para um model registrado.
fn validate_references(entries: &[&ModelEntry]) -> Result<(), String> {
    for entry in entries {
        for field in entry.fields {
            let Some(target) = field.references else {
                continue;
            };

            let exists = entries.iter().any(|other| other.name == target);

            if !exists {
                return Err(format!(
                    "O campo '{}.{}' referencia '{}', mas esse model não foi registrado com #[derive(Djanho)]",
                    entry.name, field.name, target
                ));
            }
        }
    }

    Ok(())
}

/// Coloca as tabelas referenciadas antes de quem as referencia.
/// Em caso de ciclo, mantém o restante na ordem original.
fn order_by_dependency<'a>(entries: &[&'a ModelEntry]) -> Vec<&'a ModelEntry> {
    let mut ordered: Vec<&ModelEntry> = Vec::new();
    let mut pending: Vec<&ModelEntry> = entries.to_vec();

    while !pending.is_empty() {
        let before = pending.len();
        let mut still_pending = Vec::new();

        for entry in pending {
            let ready = entry.fields.iter().all(|field| match field.references {
                None => true,
                Some(target) => {
                    target == entry.name || ordered.iter().any(|done| done.name == target)
                }
            });

            if ready {
                ordered.push(entry);
            } else {
                still_pending.push(entry);
            }
        }

        pending = still_pending;

        if pending.len() == before {
            ordered.extend(pending);
            break;
        }
    }

    ordered
}

fn generate_model_form(
    entry: &ModelEntry,
    choices: &[&ChoiceEntry],
    relations: &[ManyToManyRelation],
) -> Result<String, String> {
    let route = entry.name;
    let mut html = String::new();

    html.push_str(&format!(
        "<h1 class=\"djanho-title\"> {} </h1>\n",
        entry.name
    ));
    html.push_str(&format!(
        "<form class=\"djanho-form\" method=\"post\" action=\"/{}\">\n",
        route
    ));

    for field in entry.fields {
        let field_html = generate_field_html(entry.name, field, choices)?;

        for line in field_html.lines() {
            html.push_str("    ");
            html.push_str(line);
            html.push('\n');
        }
    }

    for relation in relations {
        if relation.left_model == entry.name {
            append_many_to_many_control(&mut html, relation, &relation.right_model);
        }
        if relation.right_model == entry.name && relation.left_model != entry.name {
            append_many_to_many_control(&mut html, relation, &relation.left_model);
        }
    }

    html.push_str("    <button class=\"djanho-submit\" type=\"submit\">Cadastrar</button>\n");
    html.push_str("</form>");

    Ok(html)
}

fn generate_field_html(
    model_name: &str,
    field: &FieldEntry,
    choices: &[&ChoiceEntry],
) -> Result<String, String> {
    let id = format!("{}-{}", model_name.to_lowercase(), field.name);
    let required = if field.optional { "" } else { " required" };

    if let Some(target) = field.references {
        return Ok(generate_foreign_key_select(
            &id,
            field.name,
            field.optional,
            target,
        ));
    }

    let html = match field.kind {
        FieldKind::Text => format!(
            "<label class=\"djanho-label\" for=\"{id}\">{label}</label>\n\
             <input class=\"djanho-input\" type=\"text\" id=\"{id}\" name=\"{name}\"{required}>",
            label = field.name,
            name = field.name,
        ),

        FieldKind::Character => format!(
            "<label class=\"djanho-label\" for=\"{id}\">{label}</label>\n\
             <input class=\"djanho-input\" type=\"text\" id=\"{id}\" name=\"{name}\" maxlength=\"1\"{required}>",
            label = field.name,
            name = field.name,
        ),

        FieldKind::Integer => format!(
            "<label class=\"djanho-label\" for=\"{id}\">{label}</label>\n\
             <input class=\"djanho-input\" type=\"number\" id=\"{id}\" name=\"{name}\" step=\"1\"{required}>",
            label = field.name,
            name = field.name,
        ),

        FieldKind::Real => format!(
            "<label class=\"djanho-label\" for=\"{id}\">{label}</label>\n\
             <input class=\"djanho-input\" type=\"number\" id=\"{id}\" name=\"{name}\" step=\"any\"{required}>",
            label = field.name,
            name = field.name,
        ),

        // bool possui dois estados. Um checkbox desmarcado não é enviado pelo
        // navegador; o backend deve interpretar ausência como false e presença
        // com value="1" como true. Não usamos `required`, pois isso obrigaria
        // o usuário a marcar o checkbox e tornaria false impossível.
        FieldKind::Boolean if !field.optional => generate_boolean_checkbox(&id, field.name),

        // Option<bool> possui três estados: None, Some(true) e Some(false).
        // Um checkbox sozinho não consegue representar os três, então usamos
        // um select opcional.
        FieldKind::Boolean => generate_optional_boolean_select(&id, field.name),

        FieldKind::Choice(choice_name) => {
            let choice = find_choice(choices, choice_name).ok_or_else(|| {
                format!(
                    "O campo '{}.{}' usa o tipo '{}', mas esse enum não foi registrado com #[derive(DjanhoChoice)]",
                    model_name, field.name, choice_name
                )
            })?;

            let options: Vec<(&str, &str)> = choice
                .variants
                .iter()
                .map(|variant| (*variant, *variant))
                .collect();

            generate_select(
                &id,
                field.name,
                field.optional,
                if field.optional {
                    "Nenhum"
                } else {
                    "Selecione..."
                },
                &options,
            )
        }
    };

    Ok(html)
}

fn append_many_to_many_control(html: &mut String, relation: &ManyToManyRelation, target: &str) {
    let id = format!("{}-{}", target.to_lowercase(), relation.name.to_lowercase());
    html.push_str(&format!(
        "    <label class=\"djanho-label\" for=\"{id}\">{target}</label>\n\
         \x20   <select class=\"djanho-select\" id=\"{id}\" name=\"__relation__{}\" \
         multiple data-djanho-target=\"{target}\">\n\
         \x20   </select>\n",
        relation.name
    ));
}

fn generate_foreign_key_select(id: &str, name: &str, optional: bool, target: &str) -> String {
    let required = if optional { "" } else { " required" };
    let mut html = format!(
        "<label class=\"djanho-label\" for=\"{id}\">{name}</label>\n\
         <select class=\"djanho-select\" id=\"{id}\" name=\"{name}\"{required} \
         data-djanho-target=\"{target}\">\n"
    );

    if optional {
        html.push_str("    <option value=\"\">Nenhum</option>\n");
    } else {
        html.push_str("    <option value=\"\" disabled selected>Selecione...</option>\n");
    }
    html.push_str("</select>");
    html
}

/// Gera o controle de um `bool` obrigatório.
///
/// Importante: `required` não deve ser usado em checkbox booleano, pois
/// significaria que o formulário só é válido quando o checkbox estiver
/// marcado. Para o Djanho, desmarcado representa `false` e marcado representa
/// `true`.
fn generate_boolean_checkbox(id: &str, name: &str) -> String {
    format!(
        "<label class=\"djanho-checkbox-label\" for=\"{id}\">\n\
         \x20   <input class=\"djanho-checkbox\" type=\"checkbox\" id=\"{id}\" name=\"{name}\" value=\"1\">\n\
         \x20   {name}\n\
         </label>"
    )
}

/// Gera o controle de `Option<bool>`.
///
/// Os valores enviados pelo formulário são:
/// - `""`  -> `None`
/// - `"1"` -> `Some(true)`
/// - `"0"` -> `Some(false)`
fn generate_optional_boolean_select(id: &str, name: &str) -> String {
    generate_select(
        id,
        name,
        true,
        "Não informado",
        &[("1", "Sim"), ("0", "Não")],
    )
}

fn generate_select(
    id: &str,
    name: &str,
    optional: bool,
    empty_label: &str,
    options: &[(&str, &str)],
) -> String {
    let required = if optional { "" } else { " required" };
    let mut html = String::new();

    html.push_str(&format!(
        "<label class=\"djanho-label\" for=\"{id}\">{name}</label>\n"
    ));
    html.push_str(&format!(
        "<select class=\"djanho-select\" id=\"{id}\" name=\"{name}\"{required}>\n"
    ));

    if optional {
        html.push_str(&format!("    <option value=\"\">{empty_label}</option>\n"));
    } else {
        html.push_str(&format!(
            "    <option value=\"\" disabled selected>{empty_label}</option>\n"
        ));
    }

    for (value, label) in options {
        html.push_str(&format!("    <option value=\"{value}\">{label}</option>\n"));
    }

    html.push_str("</select>");
    html
}

fn generate_css() -> String {
    r#"<style>
.djanho-title {
    color: #1f2937;
    font-family: system-ui, sans-serif;
    margin: 1.5rem auto 0.75rem;
    max-width: 32rem;
}

.djanho-form {
    background: #ffffff;
    border: 1px solid #d1d5db;
    border-radius: 0.5rem;
    box-shadow: 0 2px 8px rgb(0 0 0 / 6%);
    box-sizing: border-box;
    color: #1f2937;
    font-family: system-ui, sans-serif;
    margin: 0 auto 2rem;
    max-width: 32rem;
    padding: 1.5rem;
}

.djanho-label {
    display: block;
    font-weight: 600;
    margin: 1rem 0 0.375rem;
}

.djanho-input,
.djanho-select {
    background: #ffffff;
    border: 1px solid #9ca3af;
    border-radius: 0.25rem;
    box-sizing: border-box;
    color: inherit;
    font: inherit;
    padding: 0.625rem 0.75rem;
    width: 100%;
}

.djanho-input:focus,
.djanho-select:focus {
    border-color: #2563eb;
    outline: 2px solid #2563eb;
    outline-offset: 1px;
}

.djanho-checkbox-label {
    align-items: center;
    display: flex;
    font-weight: 600;
    gap: 0.5rem;
    margin: 1rem 0;
}

.djanho-checkbox {
    accent-color: #2563eb;
    height: 1rem;
    width: 1rem;
}

.djanho-submit {
    background: #2563eb;
    border: 0;
    border-radius: 0.25rem;
    color: #ffffff;
    cursor: pointer;
    font: inherit;
    font-weight: 600;
    margin-top: 1.25rem;
    padding: 0.625rem 1rem;
}

.djanho-submit:hover {
    background: #1d4ed8;
}

.djanho-submit:focus-visible {
    outline: 2px solid #1d4ed8;
    outline-offset: 2px;
}
</style>"#
        .to_string()
}

fn generate_model_schema(entry: &ModelEntry, choices: &[&ChoiceEntry]) -> Result<String, String> {
    let mut columns = vec!["    id INTEGER PRIMARY KEY".to_string()];

    for field in entry.fields {
        columns.push(generate_sql_column(entry.name, field, choices)?);
    }

    let mut sql = format!(
        "-- {}\nCREATE TABLE IF NOT EXISTS {} (\n",
        entry.name, entry.name
    );
    sql.push_str(&columns.join(",\n"));
    sql.push_str("\n);");

    Ok(sql)
}

fn generate_sql_column(
    model_name: &str,
    field: &FieldEntry,
    choices: &[&ChoiceEntry],
) -> Result<String, String> {
    let not_null = if field.optional { "" } else { " NOT NULL" };

    let mut column = match field.kind {
        FieldKind::Text => {
            format!("    {} TEXT{}", field.name, not_null)
        }

        FieldKind::Character => format!(
            "    {0} TEXT{1} CHECK (length({0}) = 1)",
            field.name, not_null
        ),

        FieldKind::Integer => {
            format!("    {} INTEGER{}", field.name, not_null)
        }

        FieldKind::Real => {
            format!("    {} REAL{}", field.name, not_null)
        }

        FieldKind::Boolean => format!(
            "    {0} BOOLEAN{1} CHECK ({0} IN (0, 1))",
            field.name, not_null
        ),

        FieldKind::Choice(choice_name) => {
            let choice = find_choice(choices, choice_name).ok_or_else(|| {
                format!(
                    "O campo '{}.{}' usa o tipo '{}', mas esse enum não foi registrado com #[derive(DjanhoChoice)]",
                    model_name, field.name, choice_name
                )
            })?;

            if choice.variants.is_empty() {
                return Err(format!("O enum '{}' não possui variantes", choice.name));
            }

            let accepted_values = choice
                .variants
                .iter()
                .map(|variant| format!("'{}'", variant))
                .collect::<Vec<_>>()
                .join(", ");

            format!(
                "    {0} TEXT{1} CHECK ({0} IN ({2}))",
                field.name, not_null, accepted_values
            )
        }
    };

    if let Some(target) = field.references {
        column.push_str(&format!(" REFERENCES {}(id)", target));
    }

    Ok(column)
}

fn find_choice<'a>(choices: &'a [&ChoiceEntry], name: &str) -> Option<&'a ChoiceEntry> {
    choices.iter().copied().find(|choice| choice.name == name)
}

/// Gera a tabela intermediária de um relacionamento N:N.
fn generate_many_to_many_schema(
    relation: &ManyToManyRelation,
    entries: &[&ModelEntry],
) -> Result<String, String> {
    let left_exists = entries
        .iter()
        .any(|entry| entry.name == relation.left_model);

    let right_exists = entries
        .iter()
        .any(|entry| entry.name == relation.right_model);

    if !left_exists {
        return Err(format!(
            "O modelo '{}' do relacionamento '{}' não foi registrado",
            relation.left_model, relation.name
        ));
    }

    if !right_exists {
        return Err(format!(
            "O modelo '{}' do relacionamento '{}' não foi registrado",
            relation.right_model, relation.name
        ));
    }

    if relation.left_field == relation.right_field {
        return Err(format!(
            "O relacionamento '{}' usa o mesmo nome de coluna nos dois lados",
            relation.name
        ));
    }

    let self_relation_check = if relation.left_model == relation.right_model {
        format!(
            ",\n    CHECK ({} < {})",
            relation.left_field, relation.right_field
        )
    } else {
        String::new()
    };

    Ok(format!(
        "-- Relacionamento N:N: {} <-> {}\n\
         CREATE TABLE IF NOT EXISTS {} (\n\
             {} INTEGER NOT NULL REFERENCES {}(id),\n\
             {} INTEGER NOT NULL REFERENCES {}(id),\n\
             PRIMARY KEY ({}, {}){}\n\
         );",
        relation.left_model,
        relation.right_model,
        relation.name,
        relation.left_field,
        relation.left_model,
        relation.right_field,
        relation.right_model,
        relation.left_field,
        relation.right_field,
        self_relation_check
    ))
}

//! Testes de integração para confirmar que os derives registram metadados
//! que podem ser encontrados pelo restante da aplicação.

use djanho_orm::Djanho;

// Os campos N:N precisam ser Vec de outro model e aparecem no registro próprio
// de relações, em vez de virar colunas dentro da tabela.
#[derive(Djanho)]
struct RelationshipStudent {
    #[djanho(many_to_many = "RelationshipCourse")]
    courses: Vec<RelationshipCourse>,
}

#[derive(Djanho)]
struct RelationshipCourse {
    #[djanho(many_to_many = "RelationshipStudent")]
    students: Vec<RelationshipStudent>,
}

#[derive(Djanho)]
struct RelationshipArticle {
    #[djanho(many_to_many = "RelationshipTag")]
    tags: Vec<RelationshipTag>,
}

#[derive(Djanho)]
struct RelationshipTag {
    label: String,
}

#[derive(Djanho)]
struct RelationshipEmployee {
    name: String,
    #[djanho(many_to_many = "RelationshipEmployee", relation = "Colleagues")]
    colleagues: Vec<RelationshipEmployee>,
}

#[derive(Djanho)]
struct RelationshipBookRoles {
    #[djanho(many_to_many = "RelationshipTag", relation = "Written")]
    written_tags: Vec<RelationshipTag>,
    #[djanho(many_to_many = "RelationshipTag", relation = "Edited")]
    edited_tags: Vec<RelationshipTag>,
}

#[derive(Djanho)]
struct RelationshipTeacher {
    name: String,
}

#[derive(Djanho)]
struct RelationshipClass {
    #[djanho(foreign_key = "RelationshipTeacher")]
    teacher_id: Option<i64>,
}

#[test]
fn derive_registers_many_to_many_metadata_in_both_directions() {
    // inventory agrega declarações feitas em diferentes structs e módulos.
    let relations: Vec<_> =
        djanho_core::inventory::iter::<djanho_core::model_entry::ManyToManyEntry>
            .into_iter()
            .filter(|relation| {
                relation.source_model.starts_with("Relationship")
                    && relation.target_model.starts_with("Relationship")
            })
            .collect();

    assert!(relations.iter().any(|relation| {
        relation.source_model == "RelationshipStudent"
            && relation.field_name == "courses"
            && relation.target_model == "RelationshipCourse"
    }));
    assert!(relations.iter().any(|relation| {
        relation.source_model == "RelationshipCourse"
            && relation.field_name == "students"
            && relation.target_model == "RelationshipStudent"
    }));
    assert!(relations.iter().any(|relation| {
        relation.source_model == "RelationshipArticle"
            && relation.field_name == "tags"
            && relation.target_model == "RelationshipTag"
    }));
    assert!(relations.iter().any(|relation| {
        relation.source_model == "RelationshipEmployee"
            && relation.field_name == "colleagues"
            && relation.target_model == "RelationshipEmployee"
            && relation.relation_name == "Colleagues"
    }));
    assert!(relations.iter().any(|relation| {
        relation.source_model == "RelationshipBookRoles" && relation.relation_name == "Written"
    }));
    assert!(relations.iter().any(|relation| {
        relation.source_model == "RelationshipBookRoles" && relation.relation_name == "Edited"
    }));
}

#[test]
fn derive_keeps_optional_one_to_many_foreign_key_as_nullable() {
    let model = djanho_core::inventory::iter::<djanho_core::model_entry::ModelEntry>
        .into_iter()
        .find(|model| model.name == "RelationshipClass")
        .expect("RelationshipClass should be registered");
    let teacher_id = model
        .fields
        .iter()
        .find(|field| field.name == "teacher_id")
        .expect("teacher_id should be registered");

    assert_eq!(teacher_id.references, Some("RelationshipTeacher"));
    assert!(teacher_id.optional);
}

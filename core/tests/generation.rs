use djanho_core::choice_entry::ChoiceEntry;
use djanho_core::generators::{generate_form, generate_schema};
use djanho_core::model_entry::{FieldEntry, FieldKind, ModelEntry};

static GENDER: ChoiceEntry = ChoiceEntry {
    name: "Gender",
    variants: &["Male", "Female", "Other"],
};

static PERSON_FIELDS: &[FieldEntry] = &[
    FieldEntry {
        name: "name",
        kind: FieldKind::Text,
        optional: false,
    },
    FieldEntry {
        name: "age",
        kind: FieldKind::Integer,
        optional: false,
    },
    FieldEntry {
        name: "cpf",
        kind: FieldKind::Text,
        optional: true,
    },
    FieldEntry {
        name: "gender",
        kind: FieldKind::Choice("Gender"),
        optional: false,
    },
];

static PERSON: ModelEntry = ModelEntry {
    name: "Person",
    fields: PERSON_FIELDS,
};

#[test]
fn enum_becomes_html_select() {
    let html = generate_form(&[&PERSON], &[&GENDER]).unwrap();

    assert!(html.contains("<select id=\"person-gender\" name=\"gender\" required>"));
    assert!(html.contains("<option value=\"Male\">Male</option>"));
    assert!(html.contains("name=\"cpf\""));
    assert!(!html.contains("name=\"cpf\" required"));
}

#[test]
fn enum_becomes_sql_check() {
    let sql = generate_schema(&[&PERSON], &[&GENDER]).unwrap();

    assert!(sql.contains("gender TEXT NOT NULL CHECK (gender IN ('Male', 'Female', 'Other'))"));
    assert!(sql.contains("cpf TEXT"));
    assert!(!sql.contains("cpf TEXT NOT NULL"));
}

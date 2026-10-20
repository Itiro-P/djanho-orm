use djanho_core::choice_entry::ChoiceEntry;
use djanho_core::generators::{ManyToManyRelation, generate_form, generate_schema};
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
        references: None,
    },
    FieldEntry {
        name: "age",
        kind: FieldKind::Integer,
        optional: false,
        references: None,
    },
    FieldEntry {
        name: "cpf",
        kind: FieldKind::Text,
        optional: true,
        references: None,
    },
    FieldEntry {
        name: "gender",
        kind: FieldKind::Choice("Gender"),
        optional: false,
        references: None,
    },
    FieldEntry {
        name: "married",
        kind: FieldKind::Boolean,
        optional: false,
        references: None,
    },
];

static AUTHOR_FIELDS: &[FieldEntry] = &[FieldEntry {
    name: "name",
    kind: FieldKind::Text,
    optional: false,
    references: None,
}];

static BOOK_FIELDS: &[FieldEntry] = &[
    FieldEntry {
        name: "title",
        kind: FieldKind::Text,
        optional: false,
        references: None,
    },
    FieldEntry {
        name: "author_id",
        kind: FieldKind::Integer,
        optional: false,
        references: Some("Author"),
    },
];

static AUTHOR: ModelEntry = ModelEntry {
    name: "Author",
    fields: AUTHOR_FIELDS,
};
static BOOK: ModelEntry = ModelEntry {
    name: "Book",
    fields: BOOK_FIELDS,
};

#[test]
fn foreign_key_becomes_references() {
    // Book vem primeiro de propósito: a ordenação deve inverter.
    let sql = generate_schema(&[&BOOK, &AUTHOR], &[], &[]).unwrap();

    assert!(sql.contains("author_id INTEGER NOT NULL REFERENCES Author(id)"));
    assert!(sql.contains("id INTEGER PRIMARY KEY"));

    let author_pos = sql.find("CREATE TABLE IF NOT EXISTS Author").unwrap();
    let book_pos = sql.find("CREATE TABLE IF NOT EXISTS Book").unwrap();
    assert!(author_pos < book_pos);
}

#[test]
fn foreign_key_to_unknown_model_is_error() {
    assert!(generate_schema(&[&BOOK], &[], &[]).is_err());
}

static PERSON: ModelEntry = ModelEntry {
    name: "Person",
    fields: PERSON_FIELDS,
};

#[test]
fn enum_becomes_html_select() {
    let html = generate_form(&[&PERSON], &[&GENDER]).unwrap();

    assert!(html.contains(
        "<select class=\"djanho-select\" id=\"person-gender\" name=\"gender\" required>"
    ));
    assert!(html.contains("<option value=\"Male\">Male</option>"));
    assert!(html.contains("name=\"cpf\""));
    assert!(!html.contains("name=\"cpf\" required"));
    assert!(html.contains("<h1 class=\"djanho-title\">"));
    assert!(html.contains("<form class=\"djanho-form\""));
    assert!(html.contains("class=\"djanho-label\""));
    assert!(html.contains("class=\"djanho-input\""));
    assert!(html.contains("class=\"djanho-checkbox-label\""));
    assert!(html.contains("class=\"djanho-checkbox\""));
    assert!(html.contains("<button class=\"djanho-submit\""));
    assert!(html.contains("<style>"));
    assert!(html.contains(".djanho-checkbox-label"));
}

#[test]
fn enum_becomes_sql_check() {
    let sql = generate_schema(&[&PERSON], &[&GENDER], &[]).unwrap();

    assert!(sql.contains("gender TEXT NOT NULL CHECK (gender IN ('Male', 'Female', 'Other'))"));
    assert!(sql.contains("cpf TEXT"));
    assert!(!sql.contains("cpf TEXT NOT NULL"));
}

#[test]
fn many_to_many_generates_join_table() {
    let relations = vec![ManyToManyRelation {
        name: "Author_Book_Link".to_string(),
        left_model: "Author".to_string(),
        left_field: "author_id".to_string(),
        right_model: "Book".to_string(),
        right_field: "book_id".to_string(),
    }];

    let sql = generate_schema(&[&AUTHOR, &BOOK], &[], &relations).unwrap();

    assert!(sql.contains("CREATE TABLE IF NOT EXISTS Author_Book_Link"));

    assert!(sql.contains("author_id INTEGER NOT NULL REFERENCES Author(id)"));

    assert!(sql.contains("book_id INTEGER NOT NULL REFERENCES Book(id)"));

    assert!(sql.contains("PRIMARY KEY (author_id, book_id)"));
}

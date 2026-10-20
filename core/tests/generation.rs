//! Testes unitários dos textos SQL e HTML produzidos pelo gerador.
//!
//! Os valores `static` abaixo simulam os metadados que as macros registram
//! durante a compilação.

use djanho_core::choice_entry::ChoiceEntry;
use djanho_core::generators::{
    ManyToManyRelation, generate_form, generate_form_with_relations, generate_schema,
    resolve_many_to_many_relations,
};
use djanho_core::model_entry::{FieldEntry, FieldKind, ManyToManyEntry, ModelEntry};

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

static OPTIONAL_BOOK_FIELDS: &[FieldEntry] = &[FieldEntry {
    name: "author_id",
    kind: FieldKind::Integer,
    optional: true,
    references: Some("Author"),
}];

static AUTHOR: ModelEntry = ModelEntry {
    name: "Author",
    fields: AUTHOR_FIELDS,
};
static BOOK: ModelEntry = ModelEntry {
    name: "Book",
    fields: BOOK_FIELDS,
};
static OPTIONAL_BOOK: ModelEntry = ModelEntry {
    name: "OptionalBook",
    fields: OPTIONAL_BOOK_FIELDS,
};
static LINK_ONLY_MODEL: ModelEntry = ModelEntry {
    name: "LinkOnlyModel",
    fields: &[],
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

#[test]
fn one_to_many_foreign_key_can_be_optional() {
    let sql = generate_schema(&[&OPTIONAL_BOOK, &AUTHOR], &[], &[]).unwrap();

    assert!(sql.contains("author_id INTEGER REFERENCES Author(id)"));
    assert!(!sql.contains("author_id INTEGER NOT NULL REFERENCES Author(id)"));

    let html = generate_form(&[&OPTIONAL_BOOK], &[]).unwrap();
    assert!(html.contains("name=\"author_id\" data-djanho-target=\"Author\""));
    assert!(!html.contains("name=\"author_id\" required"));
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
        relation_name: String::new(),
    }];

    let sql = generate_schema(&[&AUTHOR, &BOOK], &[], &relations).unwrap();

    assert!(sql.contains("CREATE TABLE IF NOT EXISTS Author_Book_Link"));

    assert!(sql.contains("author_id INTEGER NOT NULL REFERENCES Author(id)"));

    assert!(sql.contains("book_id INTEGER NOT NULL REFERENCES Book(id)"));

    assert!(sql.contains("PRIMARY KEY (author_id, book_id)"));
}

#[test]
fn many_to_many_relation_declared_on_both_models_creates_one_join_table() {
    // Declarar a relação em ambos os modelos é comum; a geração deve remover
    // a duplicata e criar uma única tabela intermediária.
    static AUTHOR_TO_BOOK: ManyToManyEntry = ManyToManyEntry {
        source_model: "Author",
        field_name: "books",
        target_model: "Book",
        relation_name: "",
    };
    static BOOK_TO_AUTHOR: ManyToManyEntry = ManyToManyEntry {
        source_model: "Book",
        field_name: "authors",
        target_model: "Author",
        relation_name: "",
    };

    let relations = resolve_many_to_many_relations(&[&AUTHOR_TO_BOOK, &BOOK_TO_AUTHOR]).unwrap();
    assert_eq!(relations.len(), 1);

    let sql = generate_schema(&[&AUTHOR, &BOOK], &[], &relations).unwrap();
    assert_eq!(
        sql.matches("CREATE TABLE IF NOT EXISTS Author_Book_Link")
            .count(),
        1
    );
}

#[test]
fn many_to_many_to_unknown_model_is_error() {
    let relation = ManyToManyRelation {
        name: "Author_Unknown_Link".to_string(),
        left_model: "Author".to_string(),
        left_field: "author_id".to_string(),
        right_model: "Unknown".to_string(),
        right_field: "unknown_id".to_string(),
        relation_name: String::new(),
    };

    assert!(generate_schema(&[&AUTHOR], &[], &[relation]).is_err());
}

#[test]
fn many_to_many_supports_self_relationships() {
    static SELF_RELATION: ManyToManyEntry = ManyToManyEntry {
        source_model: "Author",
        field_name: "coauthors",
        target_model: "Author",
        relation_name: "",
    };

    let relations = resolve_many_to_many_relations(&[&SELF_RELATION]).unwrap();
    assert_eq!(relations.len(), 1);
    assert_eq!(relations[0].name, "Author_Author_Link");
    assert_eq!(relations[0].left_field, "source_id");
    assert_eq!(relations[0].right_field, "target_id");

    let sql = generate_schema(&[&AUTHOR], &[], &relations).unwrap();
    assert!(sql.contains("source_id INTEGER NOT NULL REFERENCES Author(id)"));
    assert!(sql.contains("target_id INTEGER NOT NULL REFERENCES Author(id)"));
    assert!(sql.contains("CHECK (source_id < target_id)"));
}

#[test]
fn many_to_many_supports_multiple_named_relations_between_same_models() {
    // Nomes diferentes representam relações diferentes entre o mesmo par.
    static AUTHOR_BOOKING: ManyToManyEntry = ManyToManyEntry {
        source_model: "Author",
        field_name: "books_written",
        target_model: "Book",
        relation_name: "Written",
    };
    static AUTHOR_EDITING: ManyToManyEntry = ManyToManyEntry {
        source_model: "Author",
        field_name: "books_edited",
        target_model: "Book",
        relation_name: "Edited",
    };
    static BOOK_WRITTEN_BY: ManyToManyEntry = ManyToManyEntry {
        source_model: "Book",
        field_name: "written_by",
        target_model: "Author",
        relation_name: "Written",
    };

    let relations =
        resolve_many_to_many_relations(&[&AUTHOR_BOOKING, &BOOK_WRITTEN_BY, &AUTHOR_EDITING])
            .unwrap();
    assert_eq!(relations.len(), 2);
    assert_ne!(relations[0].name, relations[1].name);
}

#[test]
fn many_to_many_rejects_invalid_sql_identifiers() {
    let relation = ManyToManyRelation {
        name: "Author_Book_Link; DROP TABLE Author".to_string(),
        left_model: "Author".to_string(),
        left_field: "author_id".to_string(),
        right_model: "Book".to_string(),
        right_field: "book_id".to_string(),
        relation_name: String::new(),
    };

    assert!(generate_schema(&[&AUTHOR, &BOOK], &[], &[relation]).is_err());
}

#[test]
fn many_to_many_rejects_duplicate_join_table_names() {
    let first = ManyToManyRelation {
        name: "Shared_Link".to_string(),
        left_model: "Author".to_string(),
        left_field: "author_id".to_string(),
        right_model: "Book".to_string(),
        right_field: "book_id".to_string(),
        relation_name: String::new(),
    };
    let second = ManyToManyRelation {
        name: "Shared_Link".to_string(),
        left_model: "Author".to_string(),
        left_field: "author_id".to_string(),
        right_model: "Book".to_string(),
        right_field: "book_id".to_string(),
        relation_name: String::new(),
    };

    assert!(generate_schema(&[&AUTHOR, &BOOK], &[], &[first, second]).is_err());
}

#[test]
fn form_creates_a_separate_section_for_many_to_many_links() {
    static AUTHOR_TO_BOOK: ManyToManyEntry = ManyToManyEntry {
        source_model: "Author",
        field_name: "books",
        target_model: "Book",
        relation_name: "",
    };
    let relations = resolve_many_to_many_relations(&[&AUTHOR_TO_BOOK]).unwrap();

    let html = generate_form_with_relations(&[&AUTHOR, &BOOK], &[], &relations).unwrap();
    assert!(html.contains("name=\"author_id\" required data-djanho-target=\"Author\""));
    assert!(html.contains(
        "<form class=\"djanho-form\" method=\"post\" action=\"/relationships/Author_Book_Link\">"
    ));
    assert_eq!(
        html.matches("name=\"author_id\" required data-djanho-target=\"Author\"")
            .count(),
        2
    );
    assert!(html.contains("name=\"book_id\" required data-djanho-target=\"Book\""));
    assert!(html.contains(">Relacionar</button>"));
    assert!(!html.contains("__relation__Author_Book_Link"));
    assert!(!html.contains("multiple data-djanho-target"));
}

#[test]
fn relationship_only_models_receive_tables_and_forms() {
    // Um model usado apenas em uma N:N ainda precisa existir para receber os
    // vínculos na tabela intermediária.
    static LINK_ONLY_RELATION: ManyToManyEntry = ManyToManyEntry {
        source_model: "LinkOnlyModel",
        field_name: "books",
        target_model: "Book",
        relation_name: "",
    };
    let relations = resolve_many_to_many_relations(&[&LINK_ONLY_RELATION]).unwrap();

    let sql = generate_schema(&[&LINK_ONLY_MODEL, &AUTHOR, &BOOK], &[], &relations).unwrap();
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS LinkOnlyModel"));
    assert!(sql.contains("id INTEGER PRIMARY KEY"));

    let html =
        generate_form_with_relations(&[&LINK_ONLY_MODEL, &AUTHOR, &BOOK], &[], &relations).unwrap();
    assert!(html.contains("action=\"/LinkOnlyModel\""));
    assert!(html.contains("action=\"/relationships/Book_LinkOnlyModel_Link\""));
    assert!(html.contains("data-djanho-target=\"LinkOnlyModel\""));
    assert!(html.contains("data-djanho-target=\"Book\""));
}

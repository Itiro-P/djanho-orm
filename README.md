# djanho-orm
The Djanho ORM, a (not) blazing fast Object Relational Mapper which outputs to SQL and HTML form.

## Relationships

The schema generator supports one-to-many relationships through a foreign key
on the model on the "many" side. Optional foreign keys are represented by
`Option`:

```rust
#[derive(Djanho)]
struct Teacher {
    name: String,
}

#[derive(Djanho)]
struct Class {
    #[djanho(foreign_key = "Teacher")]
    teacher_id: Option<i64>,
}
```

Many-to-many relationships use a `Vec<Model>` field annotated with
`many_to_many`:

```rust
#[derive(Djanho)]
struct Student {
    #[djanho(many_to_many = "Course")]
    courses: Vec<Course>,
}
```

The referenced type and attribute value must name the same model. The relation
can be declared on one model or both; declaring both directions still
generates one join table with foreign keys and a composite primary key. A
relationship can be named to allow multiple associations between the same
models; use the same `relation` value on inverse declarations:

```rust
#[derive(Djanho)]
struct Article {
    #[djanho(many_to_many = "Person", relation = "Authors")]
    authors: Vec<Person>,
    #[djanho(many_to_many = "Person", relation = "Editors")]
    editors: Vec<Person>,
}
```

Self-referential N:N relationships are supported too, for example a
`Vec<Employee>` annotated with `many_to_many = "Employee"`. Its generated
association is symmetric when queried.

Generated forms load options from the database for foreign keys and N:N
relationships. Submitting a model form saves its record and selected links.
The HTTP backend provides:

- `GET /Parent/{id}/children/Child/foreign_key` to list 1:N children.
- `GET /Model/{id}/related/AssociationTable` to list N:N related models.

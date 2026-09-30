use crate::model_entry::ModelEntry;

use std::fs::File;
use std::io::Write;

pub fn generate_form(entry: &ModelEntry) -> std::io::Result<()> {
    let mut form_file = File::create("form.html")?;

    form_file.write_all(String::from(entry.html).as_bytes())?;

    return Ok(());
}

pub fn generate_schema(entry: &ModelEntry) -> std::io::Result<()> {
    let mut schema_file = File::create("schema.sql")?;

    schema_file.write_all(String::from(entry.sql).as_bytes())?;

    return Ok(());
}
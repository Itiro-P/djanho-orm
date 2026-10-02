/// Guarda as informações de um campo de uma estrutura
pub struct FieldMetadata {
    /// O nome do campo
    pub name: String,
    /// O tipo do campo
    pub target_type: syn::Type
}

impl FieldMetadata {
    /// Instancia um novo `FieldMetadata`
    pub fn new(name: String, target_type: syn::Type) -> Self {
        return Self {name: name, target_type: target_type};
    }

    /// Imprime um `FieldMetadata`
    pub fn print(&self) {
        let ty = &self.target_type;
        println!("Name -> {}; Type -> {}.", self.name, quote::quote!(#ty).to_string());
    }
}

/// Guarda as informações de uma estrutura
pub struct StructMetadata {
    /// O nome da estrutura
    pub name: String,
    /// Os campos da estrutura
    pub fields: Vec<FieldMetadata>
}

impl StructMetadata {
    /// Instancia um novo `StructMetadata`
    pub fn new(name: String, fields: Vec<FieldMetadata>) -> Self {
        return Self {name: name, fields: fields};
    }

    /// Imprime um `StructMetadata`
    pub fn print(&self) {
        println!("Struct: {}\nFields:", self.name);
        
        for field in &self.fields {
            print!(" - ");
            field.print();
        }
    }

    /// Gera um formulário html correspondente à estrutura
    pub fn to_html(&self) -> String {
        return String::from("html muito bonito aq :)\n");
    }
    
    /// Gera uma tabela sql correspondente à estrutura
    pub fn to_sql(&self) -> String {
        let mut columns: Vec<String> = Vec::new();
        let mut sql: String = String::new();
        sql.push_str(&format!("CREATE TABLE IF NOT EXISTS {} (\n", self.name));
        for field in &self.fields {
            let field_type = sql_type(&field.target_type);

            match field_type {
                Some(ty) => columns.push(format!("\t{} {}", field.name, ty)),
                None => continue,
            }  
        }
        
        if columns.is_empty() {
            return String::from("");
        }

        sql.push_str(&columns.join(",\n"));
        sql.push_str("\n);");

        return sql;
    }
}

/// Retorna o tipo correspondente do sql relacionado ao tipo rust dado
fn sql_type(ty: &syn::Type) -> Option<&'static str> {
    if let syn::Type::Path(type_path) = ty {
        let seg = type_path.path.segments.last()?;
        let name = seg.ident.to_string();
        match name.as_str() {
            "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "bool" => Some("INTEGER"),
            "f32" | "f64" => Some("REAL"),
            "String" | "char" => Some("TEXT"),
            _ => None,
        }
    } else {
        return None;
    }
}
pub struct FieldMetadata {
    pub name: String,
    pub target_type: syn::Type,
    pub required: bool,
}

impl FieldMetadata {
    pub fn new(name: String, target_type: syn::Type) -> Self {
        return Self {name: name, target_type: target_type, required: true};
    }

    pub fn print(&self) {
        let ty = &self.target_type;
        println!("Name -> {}; Type -> {}; Required? {}.", self.name, quote::quote!(#ty).to_string(), self.required);
    }
}

pub struct StructMetadata {
    pub name: String,
    pub fields: Vec<FieldMetadata>
}

impl StructMetadata {
    pub fn new(name: String, fields: Vec<FieldMetadata>) -> Self {
        return Self {name: name, fields: fields};
    }

    pub fn print(&self) {
        println!("Struct: {}\nFields:", self.name);
        
        for field in &self.fields {
            print!(" - ");
            field.print();
        }
    }

    pub fn to_html(&self) -> String {
        return String::from("html muito bonito aq :)\n");
    }
    
    pub fn to_sql(&self) -> String {
        return String::from("sql muito bonito aq :)\n");
    }
}
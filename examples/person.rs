use djanho_orm::Djanho;

#[derive(Djanho)]
struct Person {
    name: String,
    age: i32,
    cpf: Option<String>
}

#[derive(Djanho)]
struct Pig {
    name: String,
    age: i32,
    cpf: Option<String>
}

fn main() {  
    let _ = djanho_core::generate();
}
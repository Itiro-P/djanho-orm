use djanho_orm::{Djanho, DjanhoChoice};

#[derive(DjanhoChoice)]
enum Gender {
    Male,
    Female,
    Other,
}

#[derive(Djanho)]
struct Person {
    name: String,
    age: i32,
    cpf: Option<String>,
    gender: Gender,
    married: bool,
}

#[derive(Djanho)]
struct NotEmpty{
    something_str: String,
    something_i32: i32,
    something_f64: f64,
    something_bool: bool,
}

#[derive(Djanho)]
struct Empty {}

fn main() {
    if let Err(error) = djanho_core::generate() {
        eprintln!("Erro ao gerar arquivos do Djanho: {error}");
    }
}

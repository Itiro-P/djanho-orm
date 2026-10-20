// Exemplo executável: as derives registram os tipos e `generate` cria os
// arquivos docker/schema.sql e docker/form.html.
use djanho_orm::{Djanho, DjanhoChoice};

// Este enum será convertido em opções HTML e em uma restrição CHECK no SQL.
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
struct NotEmpty {
    something_str: String,
    something_i32: i32,
    something_f64: f64,
    something_bool: bool,
}

#[derive(Djanho)]
struct Author {
    name: String,
}

#[derive(Djanho)]
struct Book {
    title: String,
    // Esta chave estrangeira cria uma relação 1:N: um autor pode ter vários livros.
    #[djanho(foreign_key = "Author")]
    author_id: i32,
}

// Models sem campos também são permitidos, embora normalmente seja melhor
// adicionar ao menos um dado próprio.
#[derive(Djanho)]
struct Empty {}

fn main() {
    // A geração pode falhar, por exemplo, se uma FK apontar para um model
    // que não foi registrado. Nesse caso, exibimos a causa no terminal.
    if let Err(error) = djanho_core::generate() {
        eprintln!("Erro ao gerar arquivos do Djanho: {error}");
    }
}

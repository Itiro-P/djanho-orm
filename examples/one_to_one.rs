use djanho_orm::Djanho;

#[derive(Djanho)]
struct Person {
    name: String,
}

#[derive(Djanho)]
struct Profile {
    // Relação 1:1 pretendida: cada perfil pertence a uma pessoa.
    // O Djanho ainda não gera UNIQUE para a FK, então o banco não impede que
    // mais de um perfil referencie a mesma pessoa.
    #[djanho(foreign_key = "Person")]
    person_id: i64,
    biography: String,
}

fn main() {
    if let Err(error) = djanho_core::generate() {
        eprintln!("Erro ao gerar arquivos do Djanho: {error}");
    }
}

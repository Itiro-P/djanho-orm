use djanho_orm::Djanho;

#[derive(Djanho)]
struct Author {
    name: String,
}

#[derive(Djanho)]
struct Book {
    title: String,
    // Cada livro pertence a um autor; a FK permite vários livros por autor.
    #[djanho(foreign_key = "Author")]
    author_id: i64,
}

fn main() {
    if let Err(error) = djanho_core::generate() {
        eprintln!("Erro ao gerar arquivos do Djanho: {error}");
    }
}

use djanho_orm::Djanho;

#[derive(Djanho)]
struct Student {
    name: String,
    #[djanho(many_to_many = "Course")]
    courses: Vec<Course>,
}

#[derive(Djanho)]
struct Course {
    name: String,
    #[djanho(many_to_many = "Student")]
    students: Vec<Student>,
}

fn main() {
    if let Err(error) = djanho_core::generate() {
        eprintln!("Erro ao gerar arquivos do Djanho: {error}");
    }
}

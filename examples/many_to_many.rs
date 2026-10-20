use djanho_orm::{Djanho, DjanhoChoice};

#[derive(DjanhoChoice)]
enum Gender {
    Male,
    Female,
    Crab
}

#[derive(Djanho)]
struct Student {
    name: String,
    gender: Gender,
    is_veteran: bool,
    age: u32,
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

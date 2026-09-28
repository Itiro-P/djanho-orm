struct Person {
    name: String,
    age: i32,
}

fn main() {
    let irineu = Person { name: String::from("Irineu"), age: 18 };

    println!("Person's name: {}\nPerson's age: {}", irineu.name, irineu.age);
}
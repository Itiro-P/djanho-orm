# djanho-orm

O Djanho ajuda a transformar as fichas descritas no código Rust em tabelas
para o banco SQLite e formulários HTML. Por exemplo, uma ficha `Pessoa` com
nome e idade pode virar uma tabela para guardar pessoas e um formulário para
cadastrar uma.

> Apesar do nome ORM, o projeto ainda não oferece uma ferramenta Rust completa
> para consultar e editar objetos no banco. Ele gera o schema (as instruções
> para criar as tabelas), os formulários e inclui um servidor demonstrativo.

## Relacionamentos

Um relacionamento diz como fichas de tipos diferentes se conectam. Em uma
relação 1:N, por exemplo, um professor pode ter várias turmas. Cada turma
guarda o número identificador do professor:

```rust
#[derive(Djanho)]
struct Professor {
    name: String,
}

#[derive(Djanho)]
struct Turma {
    #[djanho(foreign_key = "Professor")]
    professor_id: Option<i64>,
}
```

Em uma relação N:N, vários alunos podem fazer várias disciplinas. O campo
`Vec<Disciplina>` indica que um aluno pode estar ligado a uma lista de
disciplinas:

```rust
#[derive(Djanho)]
struct Aluno {
    #[djanho(many_to_many = "Disciplina")]
    disciplinas: Vec<Disciplina>,
}
```

O tipo dentro de `Vec` e o nome em `many_to_many` precisam coincidir. O Djanho
cria uma tabela intermediária para guardar os pares de identificadores. A
relação pode ser declarada em um lado ou nos dois; se for declarada nos dois,
o Djanho ainda cria somente uma tabela. Se duas pessoas podem ter mais de um
tipo de relação com o mesmo model, use `relation` para dar nomes diferentes:

```rust
#[derive(Djanho)]
struct Livro {
    #[djanho(many_to_many = "Pessoa", relation = "Autores")]
    autores: Vec<Pessoa>,
    #[djanho(many_to_many = "Pessoa", relation = "Editores")]
    editores: Vec<Pessoa>,
}
```

Também é possível relacionar fichas do mesmo tipo, como um funcionário com
outros funcionários.

O formulário de cada ficha serve para cadastrá-la. Para cada relação N:N, o
gerador cria uma seção separada com dois seletores de fichas já cadastradas;
ao enviar, o servidor demonstrativo salva o vínculo entre elas no SQLite.

### Rotas HTTP

- `GET /Modelo/{id}/children/ModeloFilho/chave_estrangeira`: lista os
  registros filhos de uma relação 1:N.
- `GET /Modelo/{id}/related/TabelaIntermediaria`: lista os registros
  relacionados por uma associação N:N.
- `POST /relationships/TabelaIntermediaria`: associa dois registros existentes
  usando os campos definidos pela tabela intermediária.

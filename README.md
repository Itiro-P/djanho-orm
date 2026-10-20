# djanho-orm
O Djanho ORM é um mapeador objeto-relacional que gera arquivos SQL e formulários HTML.

## Relacionamentos

O gerador de esquema oferece suporte a relacionamentos um-para-muitos (1:N)
por meio de uma chave estrangeira no modelo do lado "muitos". Chaves
estrangeiras opcionais são representadas com `Option`:

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

Relacionamentos muitos-para-muitos (N:N) usam um campo `Vec<Modelo>` anotado
com `many_to_many`:

```rust
#[derive(Djanho)]
struct Aluno {
    #[djanho(many_to_many = "Disciplina")]
    disciplinas: Vec<Disciplina>,
}
```

O tipo referenciado e o valor do atributo precisam indicar o mesmo modelo. A
relação pode ser declarada em um ou nos dois modelos; declarar as duas
direções ainda gera apenas uma tabela intermediária, com chaves estrangeiras
e chave primária composta. Para criar várias associações entre o mesmo par de
modelos, dê um nome a cada relação com `relation` e use o mesmo nome nas
declarações inversas:

```rust
#[derive(Djanho)]
struct Livro {
    #[djanho(many_to_many = "Pessoa", relation = "Autores")]
    autores: Vec<Pessoa>,
    #[djanho(many_to_many = "Pessoa", relation = "Editores")]
    editores: Vec<Pessoa>,
}
```

Também há suporte a relações N:N autorreferentes. Por exemplo, um campo
`Vec<Funcionario>` pode ser anotado com
`many_to_many = "Funcionario"`. Ao consultar essa relação, a associação é
considerada simétrica.

Os formulários gerados carregam do banco as opções de chaves estrangeiras e
relacionamentos N:N. Ao enviar um formulário, o backend salva o registro e
os vínculos selecionados.

### Rotas HTTP

- `GET /Modelo/{id}/children/ModeloFilho/chave_estrangeira`: lista os
  registros filhos de uma relação 1:N.
- `GET /Modelo/{id}/related/TabelaIntermediaria`: lista os registros
  relacionados por uma associação N:N.

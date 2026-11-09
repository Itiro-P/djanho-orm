# Documentação completa do Djanho ORM

Este guia foi escrito para quem está começando. Ele explica as palavras
técnicas antes de usá-las, mostra exemplos completos e descreve o que acontece
quando o projeto é executado.

Se você nunca programou em Rust, comece pelas seções 1 e 2 e depois siga a
seção 9, “Como executar”. As seções sobre tipos e relacionamentos ajudam a
entender ou modificar os exemplos.

## Antes de começar: palavras usadas neste guia

| Palavra | Explicação simples |
|---|---|
| Programa | Instruções que o computador executa. |
| Código-fonte | O texto escrito por uma pessoa para formar um programa. Neste projeto, os arquivos Rust terminam em `.rs` e os Python em `.py`. |
| Rust | A linguagem usada para escrever as macros e gerar os arquivos do Djanho. |
| Struct | Um molde que agrupa informações de uma coisa. Por exemplo, uma struct `Pessoa` pode guardar nome e idade. |
| Campo | Uma informação dentro de uma struct. `nome: String` é um campo chamado `nome`. |
| Tipo | A categoria de um valor: texto, número, verdadeiro/falso e assim por diante. |
| Banco de dados | Um lugar organizado para guardar informações, mesmo depois que o programa fecha. |
| Tabela, linha e coluna | Pense numa planilha: a tabela é a planilha; cada linha é um registro; cada coluna é um tipo de informação. |
| SQLite | Um banco de dados que guarda tudo em um arquivo, sem precisar instalar um servidor de banco separado. |
| SQL | Uma linguagem de comandos usada para criar tabelas e consultar ou gravar dados em um banco. |
| HTML | O formato usado para descrever páginas e formulários que um navegador consegue mostrar. |
| Servidor | Um programa que fica aguardando pedidos, como alguém abrindo uma página ou enviando um formulário. |
| HTTP | O conjunto de regras que o navegador e o servidor usam para trocar pedidos e respostas pela rede. |
| JSON | Um formato de texto para devolver dados organizados; é comum em respostas de servidores. |
| Macro / `derive` | Uma ferramenta que gera código repetitivo automaticamente. Neste projeto, uma anotação informa ao Djanho como ler uma struct ou enum. |
| Compilar | Transformar o código-fonte Rust em um programa que pode ser executado. O compilador também encontra vários erros antes da execução. |
| Cargo | A ferramenta do Rust que compila o projeto, executa testes e cuida das bibliotecas usadas por ele. |
| Docker | Uma ferramenta que inicia programas dentro de um ambiente preparado, chamado container, para facilitar a execução. |
| ORM | Sigla em inglês para uma ferramenta que ajuda a relacionar estruturas do programa com tabelas do banco. O nome não significa que toda essa automação já exista neste projeto. |

Você não precisa decorar esses termos. Use esta tabela como referência; as
seções seguintes explicam como cada peça participa do projeto.

## Rust mínimo para ler os exemplos

Você não precisa conhecer Rust para executar os exemplos, mas estas regras
ajudam a entender o que está escrito:

```rust
use djanho_orm::Djanho;

#[derive(Djanho)]
struct Pessoa {
    nome: String,
}
```

- `use ...;` importa um nome de outro pacote para que ele possa ser usado no
  arquivo. O ponto e vírgula encerra essa instrução.
- `struct Pessoa { ... }` declara um tipo de dado chamado `Pessoa`. As chaves
  contêm seus campos; `nome: String` significa que o campo `nome` guarda texto.
- `#[derive(Djanho)]` é um atributo: uma instrução para o compilador. O derive
  `Djanho` lê a struct durante a compilação e registra seus campos para a
  geração posterior.
- `String` é texto que pode ter vários caracteres. O nome do tipo vem depois
  dos dois-pontos de cada campo.
- `fn main() { ... }` é a função inicial de um programa executável. As chaves
  delimitam o que será executado.
- `let` cria uma variável. Rust costuma exigir que ela não mude; quando
  precisa mudar, o código escreve `let mut`.
- `Option<T>` representa um valor que pode estar presente (`Some(valor)`) ou
  ausente (`None`). `Vec<T>` representa uma lista de valores do tipo `T`.
- `Result<T, E>` representa uma operação que pode dar certo (`Ok(valor)`) ou
  falhar (`Err(erro)`). Por isso os exemplos verificam o resultado da geração
  em vez de esconder um erro.
- O compilador verifica tipos, nomes e regras antes de executar o programa.
  Uma mensagem de compilação normalmente aponta a linha e a coluna do
  problema; ela não significa que o programa chegou a iniciar.

Rust não executa o derive enquanto o programa está rodando. O derive produz
registros que ficam no programa compilado; depois, `generate()` lê esses
registros e escreve os arquivos.

## 1. O que é o Djanho ORM

Imagine que você está criando um sistema para guardar pessoas e livros. Você
precisa decidir quais informações cada pessoa tem, como guardar isso numa
tabela e como permitir que alguém preencha essas informações. Normalmente,
seria necessário escrever essas partes manualmente.

O Djanho automatiza uma parte desse trabalho. Você descreve os dados em
estruturas Rust e o projeto usa essa descrição para gerar:

- um schema SQL compatível com SQLite;
- formulários HTML para cadastrar registros;
- um exemplo de servidor HTTP em Python que recebe os formulários, grava dados
  no SQLite e permite consultar registros e relacionamentos.

Em termos simples: você descreve os dados uma vez no código Rust, e o Djanho
usa essa descrição para preparar as tabelas e os formulários.

**Importante:** apesar do nome ORM, o Djanho ainda não faz toda a tarefa de um
ORM completo. Ele gera a estrutura do banco e os formulários, e inclui um
servidor demonstrativo em Python. Ele ainda não permite que o programa Rust
carregue, consulte e grave automaticamente valores em structs.

## 2. Visão geral do fluxo

1. Você escreve as informações que quer guardar numa `struct` Rust. Uma
   `struct` é como o desenho de uma ficha: diz que uma pessoa tem nome, idade
   etc.
2. Você coloca `#[derive(Djanho)]` acima da struct. Essa anotação pede ao Rust
   para executar o código auxiliar do Djanho e registrar como a ficha deve ser
   representada no banco e no formulário.
3. Durante a compilação, as macros do projeto leem essa descrição. Uma macro
   é uma ferramenta que automatiza a escrita de código repetitivo.
4. O exemplo chama `djanho_core::generate()`. Essa função junta as descrições
   e cria dois arquivos: um com instruções para o banco e outro com os
   formulários.
5. O servidor demonstrativo em Python usa esses arquivos. Quando alguém envia
   um formulário, ele grava as informações no SQLite, que as mantém num
   arquivo de banco de dados.

```text
Você descreve os dados
          │
          ▼
O Djanho lê as descrições
          │
          ▼
Ele cria as instruções do banco e os formulários
          │
          ▼
O servidor Python usa esses arquivos e grava dados no SQLite
```

As palavras mais técnicas no desenho significam:

- **`schema.sql`**: arquivo de instruções que cria as tabelas e suas regras no
  banco.
- **`form.html`**: arquivo com os campos que aparecem na página para digitar
  informações.
- **`backend`**: palavra em inglês para a parte do sistema que trabalha nos
  bastidores. Aqui, é o servidor Python que recebe os formulários e conversa
  com o banco.
- **metadados**: informações sobre outras informações. Por exemplo, o nome
  `Pessoa`, o campo `idade` e o fato de `idade` ser um número são metadados.

## 3. Organização do repositório

```text
.
├── Cargo.toml                 # Pacote principal e configuração do workspace Rust
├── Cargo.lock                 # Versões resolvidas das dependências Rust
├── README.md                  # Apresentação curta e exemplos de relacionamentos
├── DOCUMENTACAO.md            # Este guia detalhado
├── LICENSE                    # Licença MIT
├── src/
│   └── lib.rs                 # Reexporta as macros para os utilizadores
├── core/
│   ├── Cargo.toml             # Dependências do núcleo
│   ├── src/
│   │   ├── lib.rs             # Registros globais e função generate()
│   │   ├── choice_entry.rs    # Metadados de enums de opções
│   │   ├── model_entry.rs     # Metadados de models, campos e N:N
│   │   ├── metadata.rs        # Análise de tipos Rust
│   │   └── generators.rs      # Geração e validação de SQL e HTML
│   └── tests/
│       └── generation.rs      # Testes de SQL, HTML e relacionamentos
├── macros/
│   ├── Cargo.toml             # Configuração da crate de macros procedurais
│   └── src/
│       └── lib.rs             # Implementa Djanho e DjanhoChoice
├── examples/
│   ├── one_to_one.rs          # Exemplo com Person e Profile
│   ├── one_to_many.rs         # Exemplo com Author e Book
│   └── many_to_many.rs        # Exemplo com Student e Course
├── tests/
│   └── lib.rs                 # Confere o registro de metadados pelos derives
└── docker/
    ├── app.py                 # Servidor HTTP e integração com SQLite
    ├── Dockerfile             # Ambiente Python e ferramenta SQLite
    ├── docker-compose.yml     # Portas, arquivos montados e volume do banco
    └── test_app.py            # Testes do servidor, rotas e operações SQLite
```

Os arquivos `docker/schema.sql` e `docker/form.html` são criados quando você
executa o exemplo. Eles não ficam guardados no Git, então precisam ser gerados
antes de iniciar o serviço Docker.

### Para que serve cada parte

| Arquivo ou pasta | O que faz |
|---|---|
| `Cargo.toml` | Identifica o pacote principal, escolhe a edição Rust 2024, liga os pacotes locais `core` e `macros` e declara o workspace. |
| `Cargo.lock` | Registra versões exatas das bibliotecas externas resolvidas pelo Cargo; ajuda a repetir a mesma compilação. Não é código escrito para a lógica do Djanho. |
| `src/lib.rs` | É a entrada da biblioteca pública. Reexporta os derives para permitir `use djanho_orm::Djanho`. |
| `macros/src/lib.rs` | Lê structs, enums e atributos `#[djanho(...)]`; valida o formato e registra descrições estáticas. |
| `core/src/metadata.rs` | Interpreta tipos Rust, como `String`, `Option<i64>` e enums, para classificá-los. |
| `core/src/model_entry.rs` | Define os formatos dos metadados de models, campos e relações N:N. |
| `core/src/choice_entry.rs` | Define os metadados de um enum usado como lista de opções. |
| `core/src/generators.rs` | Valida referências e transforma metadados em texto SQL e HTML; não abre o banco de dados. |
| `core/src/lib.rs` | Reúne os registros das macros, coordena os geradores e grava os dois arquivos em `docker/`. |
| `examples/*.rs` | São programas Rust independentes. Cada um descreve um conjunto de models e chama a geração. |
| `core/tests/generation.rs` | Testa diretamente os textos SQL/HTML e as regras de relacionamento do núcleo. |
| `tests/lib.rs` | Testa se os derives registraram os metadados que o núcleo espera encontrar. |
| `docker/app.py` | Lê o schema, serve o formulário, recebe dados, grava e consulta o SQLite. |
| `docker/test_app.py` | Testa as operações do backend com bancos temporários e também exercita rotas HTTP. |
| `docker/Dockerfile` e `docker/docker-compose.yml` | Preparam e iniciam o servidor Python, encaminham a porta e guardam o banco num volume. |
| `README.md` | Apresentação rápida do projeto; este documento explica o funcionamento com mais detalhes. |
| `LICENSE` | Informa as condições da licença MIT. |
| `.gitignore` | Evita versionar arquivos compilados e os arquivos gerados `schema.sql` e `form.html`. |

## 4. Os componentes Rust

### Pacote principal

`src/lib.rs` deixa `Djanho` e `DjanhoChoice` disponíveis para o restante do
projeto. Assim, o exemplo pode importá-los por um único caminho:

```rust
use djanho_orm::{Djanho, DjanhoChoice};
```

O pacote principal usa duas partes internas: `djanho-macros`, que lê as
anotações, e `djanho-core`, que transforma as descrições em arquivos. Uma
**crate** é, em termos simples, um pacote de código Rust. Um **workspace** é
um conjunto de pacotes que Cargo compila e testa juntos.

### `djanho-macros`

É o pacote que contém as macros. Ele usa bibliotecas auxiliares para ler o
código Rust e produzir os pequenos trechos de código que registram as
descrições.

- `Djanho` aceita structs com campos nomeados. Cada struct representa um tipo
  de ficha, como Pessoa ou Livro; neste projeto ela também é chamada de
  **model**.
- `DjanhoChoice` aceita enums simples. Um enum é uma lista fechada de opções,
  como `Pendente`, `EmAndamento` e `Concluido`.
- As anotações de relacionamento são conferidas quando o código é compilado.
  Se estiverem escritas de um jeito que o Djanho não entende, o compilador
  informa o erro.
- Relações N:N ficam registradas à parte porque precisam de uma tabela extra
  para conectar os dois tipos de ficha.

### `djanho-core`

O núcleo é a parte que reúne as descrições, confere se fazem sentido e escreve
os arquivos finais. A biblioteca `inventory` serve como uma lista comum:
cada macro coloca nela as informações que encontrou e, depois, o núcleo
consulta essa lista sem exigir que você cadastre cada model manualmente.

Quando o exemplo chama `djanho_core::generate()`, a função:

1. reúne models, enums e declarações N:N;
2. ordena os registros para que a saída seja estável entre execuções;
3. resolve as declarações N:N em tabelas intermediárias;
4. valida as referências e gera HTML e SQL em memória;
5. grava os arquivos no diretório `docker/`.

Se algum tipo ou relacionamento estiver errado, a função informa o problema
em vez de fingir que os arquivos foram gerados corretamente. O exemplo mostra
essa mensagem no terminal.

### Como o código passa de uma etapa para outra

1. O compilador transforma o texto da struct numa representação que a macro
   consegue percorrer. A biblioteca `syn` faz essa leitura estruturada.
2. A macro verifica se recebeu uma struct de campos nomeados, lê cada tipo e
   interpreta atributos como `foreign_key` e `many_to_many`.
3. A biblioteca `quote` monta o código Rust que registra os metadados. A
   biblioteca `inventory` guarda esses registros no programa compilado.
4. Quando `main` chama `generate()`, o núcleo reúne os registros. A função
   valida models e relações antes de solicitar a geração do SQL e do HTML.
5. Só depois que ambos os textos foram preparados, o núcleo grava
   `docker/form.html` e `docker/schema.sql`. Erros de validação são devolvidos
   como `Result`; nos exemplos, `if let Err(error)` imprime a mensagem.

Há dois níveis de validação: erros de sintaxe ou de tipo da macro aparecem ao
compilar; erros que dependem do conjunto completo de models aparecem quando
`generate()` é executada. Por exemplo, um tipo de campo não suportado pode
ser identificado durante a compilação, mas um destino de chave estrangeira
que nunca foi registrado só pode ser confirmado quando o núcleo reúne os
models.

### Funções importantes dentro do código Rust

Esta tabela funciona como um mapa para quem quiser abrir os arquivos-fonte:

| Arquivo | Função ou elemento | Papel no programa |
|---|---|---|
| `macros/src/lib.rs` | `parse_relationships` | Lê os valores escritos em `#[djanho(...)]` e rejeita atributos duplicados ou desconhecidos. |
| `macros/src/lib.rs` | `validate_many_to_many_type` | Confirma que uma relação N:N foi escrita como `Vec<Modelo>` e que o modelo do `Vec` coincide com o atributo. |
| `macros/src/lib.rs` | `djanho_derive` | Valida uma struct, separa campos comuns de relações N:N e registra seus metadados. |
| `macros/src/lib.rs` | `djanho_choice_derive` | Valida um enum simples e registra os nomes de suas opções. |
| `core/src/metadata.rs` | `analyze_type` | Descobre o tipo básico e se o campo aceita ausência. |
| `core/src/metadata.rs` | `unwrap_option` e `classify_base_type` | Reconhece `Option<T>` e converte os tipos Rust suportados nas categorias internas do Djanho. |
| `core/src/model_entry.rs` | `FieldEntry`, `ModelEntry`, `ManyToManyEntry` | São os formatos que guardam os nomes, tipos, referências e relações após a macro analisá-los. |
| `core/src/choice_entry.rs` | `ChoiceEntry` | Guarda o nome do enum e os nomes das variantes que serão as opções permitidas. |
| `core/src/generators.rs` | `resolve_many_to_many_relations` | Deduplica relações declaradas nos dois lados e escolhe nomes de tabelas e colunas intermediárias. |
| `core/src/generators.rs` | `generate_form_with_relations` | Monta os formulários dos models e os formulários separados para associações N:N. |
| `core/src/generators.rs` | `generate_field_html` | Escolhe o controle HTML conforme o tipo: texto, número, checkbox, enum ou seletor de chave estrangeira. |
| `core/src/generators.rs` | `generate_schema` | Valida models e relações e monta o SQL das tabelas e associações. |
| `core/src/generators.rs` | `generate_sql_column` | Converte um campo em coluna SQL e acrescenta nulabilidade, validações e chave estrangeira. |
| `core/src/generators.rs` | `generate_many_to_many_schema` | Cria a tabela de ligação com duas referências e uma chave primária composta. |
| `core/src/lib.rs` | `generate` | Reúne os registros, chama as funções geradoras e grava os arquivos no diretório `docker/`. |
| `src/lib.rs` | `pub use` | Torna os derives acessíveis pelo nome público `djanho_orm`. |

## 5. Tipos Rust e sua representação

Cada campo tem um tipo: isso informa que espécie de valor ele guarda. Por
exemplo, nome é texto, idade é número e ativo é verdadeiro ou falso. A tabela
abaixo mostra como o Djanho transforma cada tipo Rust em uma coluna do SQLite
e em um campo de formulário:

| Tipo Rust | SQLite | Controle HTML |
|---|---|---|
| `String` | `TEXT` | Campo de texto |
| `char` | `TEXT` com validação de um caractere | Campo de texto com `maxlength="1"` |
| Inteiros: `i8`, `i16`, `i32`, `i64`, `isize`, `u8`, `u16`, `u32`, `u64`, `usize` | `INTEGER` | Campo numérico inteiro |
| `f32`, `f64` | `REAL` | Campo numérico decimal |
| `bool` | `BOOLEAN` com restrição para `0` ou `1` | Checkbox |
| Enum com `DjanhoChoice` | `TEXT` com restrição `CHECK` | Lista de opções |
| `Option<T>` | Mesmo tipo SQL de `T`, sem `NOT NULL` | Controle opcional |

O campo `id` é criado automaticamente. Ele é um número que identifica cada
linha sem confundir uma pessoa com outra, mesmo que duas tenham o mesmo nome.
Em bancos de dados, esse identificador é chamado de **chave primária**.
Portanto, uma struct marcada com `Djanho` não pode declarar um campo chamado
`id`.

Campos obrigatórios precisam ter um valor. O Djanho registra essa regra no
banco e, quando possível, também impede que o formulário seja enviado vazio.
Em SQL, a regra aparece como `NOT NULL`.

`Option<T>` é a forma de Rust dizer “este valor pode existir ou pode estar
ausente”. Por exemplo, `Option<String>` significa que pode haver um texto ou
nenhum texto. O Djanho transforma essa ausência em um valor vazio permitido no
banco e no formulário.

Um enum de opções deve ser declarado assim:

```rust
#[derive(DjanhoChoice)]
enum Status {
    Pending,
    Active,
    Archived,
}
```

Cada variante vira uma opção textual do formulário e um valor permitido no
banco. A regra que limita os valores possíveis é chamada de **restrição**:
por exemplo, o banco aceita `Pending`, mas rejeita um texto que não esteja na
lista. Variantes com informações extras, enums vazios e tipos não
reconhecidos como um enum registrado não são aceitos.

## 6. Declarar models e relacionamentos

### Ficha simples (model)

```rust
#[derive(Djanho)]
struct Pessoa {
    nome: String,
    idade: u32,
    apelido: Option<String>,
    ativo: bool,
}
```

Aqui, `Pessoa` é um molde para uma ficha de pessoa. `nome`, `idade`, `apelido`
e `ativo` são os dados que cada ficha pode ter. O Djanho usa esse molde para
criar uma tabela `Pessoa`: cada ficha salva vira uma linha, e cada campo vira
uma coluna. Também cria a coluna `id` para distinguir as fichas.

O Djanho usa o nome da struct como nome da tabela e como parte dos endereços
do servidor. Por isso, nomes simples e consistentes ajudam a evitar confusão.

### Relação 1:N

“Um para muitos” (1:N) quer dizer que um registro de um tipo pode estar ligado
a vários registros de outro tipo. Por exemplo: um professor pode dar várias
turmas. Cada turma, por sua vez, guarda o número do professor responsável.

Esse número que aponta para outra ficha é chamado de **chave estrangeira**.
Ela funciona como uma referência: em vez de copiar todos os dados do
professor para cada turma, a turma guarda o `id` do professor.

```rust
#[derive(Djanho)]
struct Professor {
    nome: String,
}

#[derive(Djanho)]
struct Turma {
    nome: String,
    #[djanho(foreign_key = "Professor")]
    professor_id: i64,
}
```

No exemplo, `foreign_key = "Professor"` diz que `professor_id` precisa
corresponder ao `id` de uma ficha Professor. O campo precisa ser um número
inteiro reconhecido pelo Djanho, e Professor também precisa estar marcado com
`Djanho`. O nome do campo costuma terminar em `_id` para ficar claro que guarda
um identificador.

Se algumas turmas ainda não tiverem professor, use `Option<i64>`:

```rust
#[djanho(foreign_key = "Professor")]
professor_id: Option<i64>,
```

`i64` é um tipo de número inteiro do Rust; aqui ele guarda o identificador do
professor. O banco permitirá que `professor_id` fique vazio. No formulário, o
servidor mostra uma lista dos professores já cadastrados para a pessoa escolher.
Quando quiser descobrir as turmas de um professor, use a rota de consulta
descrita mais adiante.

### Relação N:N

“Muitos para muitos” (N:N) quer dizer que vários registros de um tipo podem
estar ligados a vários registros de outro tipo. Por exemplo, um aluno pode
fazer várias disciplinas e uma disciplina pode ter vários alunos.

Para guardar todos esses pares sem repetir os dados, o banco usa uma terceira
tabela, chamada **tabela intermediária**. Ela guarda pares de identificadores:
um `id` de aluno e um `id` de disciplina. O Djanho cria essa tabela a partir
de um campo `Vec<Modelo>` anotado com `many_to_many`:

```rust
#[derive(Djanho)]
struct Aluno {
    nome: String,
    #[djanho(many_to_many = "Disciplina")]
    disciplinas: Vec<Disciplina>,
}

#[derive(Djanho)]
struct Disciplina {
    nome: String,
}
```

`Vec<Disciplina>` significa uma lista de disciplinas. Aqui, ela informa ao
Djanho que existe uma relação com Disciplina. O tipo dentro de `Vec` precisa
ter o mesmo nome que o texto em `many_to_many`.

Esse campo é uma instrução para o gerador, não uma coluna que guarda uma lista
de texto. O backend Python também não cria objetos Rust: ele grava os pares de
IDs na tabela intermediária.

O núcleo cria a tabela intermediária com duas chaves estrangeiras. Para evitar
que o mesmo par aluno-disciplina seja gravado duas vezes, os dois IDs formam
juntos um identificador único, chamado **chave primária composta**. Você pode
declarar a relação em um dos models; se também declarar no outro, o Djanho
reconhece que é a mesma relação e não cria outra tabela.

Às vezes, os mesmos tipos têm mais de uma ligação. Por exemplo, uma pessoa
pode ser autora de um livro e também editora. O atributo `relation` dá um nome
a cada ligação para que elas não sejam confundidas:

```rust
#[derive(Djanho)]
struct Livro {
    #[djanho(many_to_many = "Pessoa", relation = "Autores")]
    autores: Vec<Pessoa>,
    #[djanho(many_to_many = "Pessoa", relation = "Editores")]
    editores: Vec<Pessoa>,
}

#[derive(Djanho)]
struct Pessoa {
    nome: String,
}
```

Se você escrever a mesma ligação nos dois lados, use o mesmo nome em
`relation`. O nome aparece na identificação da tabela intermediária.

Um tipo também pode se relacionar com outro registro do mesmo tipo. Por
exemplo, um funcionário pode estar ligado a outros funcionários. Isso é
chamado de autorrelação. O Djanho diferencia os dois IDs e trata a ligação
como simétrica: se A está ligado a B, a consulta também encontra A quando
parte de B. Um registro não pode ser ligado a si mesmo.

### O que o exemplo `one_to_one.rs` realmente demonstra

O arquivo [examples/one_to_one.rs](./examples/one_to_one.rs) declara `Person`
e `Profile`, e o campo `person_id` referencia `Person`. Isso cria uma chave
estrangeira comum: cada perfil precisa apontar para uma pessoa, mas o schema
atual não cria uma regra `UNIQUE` em `person_id`. Portanto, o banco permite
que vários perfis apontem para a mesma pessoa; tecnicamente, o exemplo
implementa uma relação 1:N, não uma relação 1:1 garantida.

O nome do arquivo é mantido para apresentar a intenção do exemplo, mas não
deve ser interpretado como garantia do banco. Uma relação 1:1 real exige uma
restrição de unicidade na chave estrangeira, funcionalidade que ainda não é
oferecida pela anotação atual.

## 7. Arquivos gerados

### `docker/schema.sql`: instruções para criar o banco

Este arquivo contém comandos SQL para criar as tabelas. Por exemplo,
`CREATE TABLE` significa “crie uma tabela”. O arquivo é feito para o SQLite e
inclui:

- `id INTEGER PRIMARY KEY` gerado pelo Djanho;
- tipos de coluna e nulabilidade;
- restrições para `char`, `bool` e enums;
- referências de chaves estrangeiras (IDs que apontam para outras tabelas);
- tabelas intermediárias N:N que guardam os pares de IDs relacionados.

Quando possível, o arquivo cria primeiro as tabelas que outras tabelas
precisam referenciar. O comando `IF NOT EXISTS` significa “crie somente se
ainda não existir”. Portanto, o Djanho não atualiza tabelas antigas quando
você muda uma struct. Esse processo de atualizar um banco existente é chamado
de **migração**, e o projeto ainda não o automatiza. Durante testes, você pode
apagar e recriar o banco; para dados importantes, não faça isso sem uma cópia
de segurança.

### `docker/form.html`: os formulários

Este arquivo contém os campos que o navegador mostra para cadastrar cada
tipo de ficha. Por exemplo, uma ficha Pessoa com campo `nome` terá um espaço
para digitar o nome. O arquivo também inclui estilos visuais simples.

Ele não é um site completo: não tem páginas de navegação ou edição. É o
formulário que o servidor demonstrativo apresenta.

Antes de mostrar a página, o servidor Python preenche as listas usadas para
escolher uma chave estrangeira ou uma relação N:N. Para identificar cada
opção, ele procura uma coluna chamada `name`, `title` ou `label`, nessa ordem.
Se a tabela não tiver nenhuma delas, mostra o `id`.

## 8. Backend demonstrativo em Python

`docker/app.py` é um programa Python que faz o papel de servidor. Ele recebe
pedidos do navegador, mostra o formulário e conversa com o banco SQLite. Ao
iniciar, lê `schema.sql` para criar as tabelas e abre o servidor na porta
`8000` dentro do Docker. Os dados são guardados em `/data/db.sqlite`.

### Rotas disponíveis

Uma **rota** é um endereço que o servidor reconhece. Por exemplo, quando você
abre `/form`, está pedindo a página do formulário. `{id}` nos exemplos abaixo
é um espaço reservado: substitua-o pelo número real de uma ficha, como `1`.

| Método e rota | Resultado |
|---|---|
| `GET /form` | Mostra os formulários e as opções de relações disponíveis. `GET` significa que o navegador está pedindo para receber uma página. |
| `POST /NomeDoModel` | Recebe os campos preenchidos, salva uma ficha e volta para `/form`. `POST` significa que o navegador está enviando dados ao servidor. |
| `GET /NomeDaTabela` | Devolve as fichas daquela tabela num texto organizado em JSON. |
| `GET /Professor/1/children/Turma/professor_id` | Exemplo: devolve as turmas cujo campo `professor_id` aponta para o professor de ID `1`. |
| `GET /Aluno/1/related/Aluno_Disciplina_Link` | Exemplo: devolve as disciplinas ligadas ao aluno de ID `1` pela tabela intermediária indicada. |
| `POST /relationships/NomeDaTabelaDeLigacao` | Recebe os IDs de dois registros existentes e cria o vínculo N:N entre eles. |

Os endereços de consulta devolvem JSON, um formato de texto que outros
programas conseguem ler. O navegador mostra o formulário, mas essas rotas de
consulta são mais úteis para testar ou conectar outro programa.

Ao enviar o formulário de um model, os campos comuns são gravados nas colunas
da tabela. O formulário N:N é separado: ele mostra dois seletores, um para
cada tipo relacionado, e envia os IDs escolhidos para
`/relationships/NomeDaTabelaDeLigacao`. Os dois registros precisam já existir.
O servidor grava cada inserção dentro de uma transação SQLite; se a gravação
falhar, a transação não é confirmada.

O servidor confere as referências no banco ao gravar. Também trata os valores
digitados como dados, e não como pedaços de comandos SQL. Isso é importante:
misturar texto digitado diretamente em comandos de banco pode causar erros e
problemas de segurança.

### Como as funções do servidor se encaixam

O arquivo Python usa a biblioteca padrão do Python; não é necessário instalar
um pacote Python adicional para executar o backend ou seus testes. As funções
estão organizadas assim:

| Função ou grupo | O que faz |
|---|---|
| `check_table_exists`, `get_columns` | Consulta o próprio SQLite para conferir se a tabela existe e descobrir suas colunas. |
| `quote_identifier`, `build_query` | Delimita nomes de tabela/coluna e monta instruções de inserção com valores passados separadamente, sem concatenar texto digitado ao SQL. |
| `get_link_foreign_keys` | Confere se uma tabela tem o formato de uma associação N:N, com duas chaves estrangeiras e chave composta. |
| `insert_record` | Insere um model e confirma a transação SQLite; valida os valores de campos e aceita vínculos N:N quando recebidos. |
| `insert_relationship` | Insere um vínculo N:N entre dois registros que já existem. |
| `populate_form_options` | Lê o HTML e preenche os seletores relacionais com IDs existentes e rótulos escapados para HTML. |
| `fetch_rows`, `fetch_children`, `fetch_related` | Lê linhas, filhos de uma relação 1:N ou registros associados por N:N e os devolve como dados que podem ser convertidos em JSON. |
| `init_db` | Executa `schema.sql` no arquivo SQLite na inicialização do servidor. |
| `Handler.do_GET` | Atende o formulário e as rotas de leitura; retorna status HTTP e JSON quando apropriado. |
| `Handler.do_POST` | Lê os dados enviados pelo navegador, cadastra um model ou associação e redireciona ao formulário. Erros de dados retornam status `400`; caminhos não reconhecidos retornam `404`. |
| `Handler._send`, `Handler._status` | Envia respostas HTTP com cabeçalhos e corpo, ou sem corpo. |
| Bloco `if __name__ == "__main__"` | Inicializa o banco e inicia o servidor HTTP na porta `8000`. |

### Persistência e portas no Docker

O Docker executa o servidor num ambiente separado do restante do computador.
O arquivo `docker-compose.yml` configura esse ambiente:

- o servidor escuta na porta `8000` dentro do ambiente;
- a porta `8080` do seu computador encaminha os pedidos para o servidor;
- o banco fica num espaço persistente chamado `db-data`.

Esse espaço mantém os dados se o servidor for reiniciado. O comando
`docker compose down -v` apaga também o espaço persistente e, com ele, os
dados salvos. Use-o somente se quiser realmente apagar esses dados.

### O que fazer quando a página abrir

O formulário reúne os campos dos models que o exemplo registrou. Para testar
uma relação, salve primeiro a ficha que será escolhida depois. Por exemplo,
para associar uma turma a um professor:

1. preencha e envie o formulário de Professor;
2. depois, no formulário de Turma, escolha esse professor na lista;
3. envie a turma.

Para testar N:N, salve primeiro pelo menos um registro de cada tipo. Em
seguida, use o formulário separado “Relacionar ...”, escolha um registro em
cada lista e envie. Se uma lista estiver vazia, ainda não há registros daquele
tipo disponíveis para escolher. A associação não é incluída automaticamente
ao cadastrar um model.

## 9. Como executar

### O que instalar antes

- **Rust e Cargo**: para compilar o exemplo e executar os testes Rust;
- **Python 3**: para executar os testes do servidor no próprio computador;
- **Docker e `docker compose`**: para iniciar o servidor demonstrativo em um
  ambiente preparado.

Se você nunca usou Rust: Cargo é o comando que organiza as tarefas do Rust.
Quando digitamos um comando abaixo, abra o terminal (a janela em que você
digita instruções) na pasta principal do projeto.

O projeto usa a edição 2024 do Rust. Se uma versão antiga de Rust reclamar da
edição, atualize o Rust antes de tentar novamente.

### Gerar SQL e formulário

Na raiz do repositório:

```bash
cargo run --example one_to_many
```

Esse comando pede ao Cargo para compilar e executar o exemplo `one_to_many`,
que descreve `Author` e `Book`, e chama a função que cria os arquivos.
Os exemplos disponíveis são `one_to_one`, `one_to_many` e `many_to_many`;
cada um é um programa independente e gera arquivos com apenas os models
declarados nele. Executar outro exemplo substitui os dois arquivos gerados.
Confira se apareceram:

```text
docker/schema.sql
docker/form.html
```

Esses arquivos precisam existir antes de iniciar o Compose, pois são montados
como volumes do container.

### Iniciar o servidor de demonstração

Na raiz do repositório:

```bash
docker compose -f docker/docker-compose.yml up --build
```

Esse comando monta o ambiente Docker e inicia o servidor. Enquanto ele estiver
rodando, abra [http://localhost:8080/form](http://localhost:8080/form) no
navegador para preencher os formulários. Para encerrar o servidor, volte ao
terminal onde ele está rodando e pressione `Ctrl+C`.

### Executar os testes

```bash
cargo test --workspace
python3 -m unittest discover -s docker -p 'test_*.py' -v
cargo fmt --all -- --check
```

Os testes são verificações automáticas: eles executam exemplos conhecidos e
confirmam que o resultado é o esperado.

- `cargo test --workspace` compila e executa os testes de todas as partes Rust.
- `python3 -m unittest discover -s docker -p 'test_*.py' -v` verifica o
  servidor Python e o banco.
- `cargo fmt --all -- --check` confere se o código Rust está formatado de
  maneira consistente; não altera os arquivos.

Os testes cobrem a criação do SQL e do HTML, os relacionamentos, o envio dos
formulários e as consultas ao banco.

## 10. Diagnóstico de erros comuns

- **O comando não cria os arquivos:** confirme que executou
  um exemplo existente, como `cargo run --example one_to_many`, na pasta
  principal. O programa precisa terminar sem mostrar uma mensagem de erro.
- **O Docker não encontra `schema.sql` ou `form.html`:** gere os arquivos
  primeiro e confira se estão dentro de `docker/`.
- **Um tipo de campo não é reconhecido:** use um dos tipos da tabela de tipos
  ou, se for uma lista fechada de opções, registre o enum com `DjanhoChoice`.
- **Erro ao declarar uma FK:** confira se o destino tem `Djanho`, se o nome no
  atributo é idêntico ao nome da struct e se o campo FK é inteiro.
- **Erro ao declarar N:N:** confira se o campo é `Vec<Modelo>` e se o nome do
  tipo e o texto em `many_to_many` coincidem.
- **O exemplo `one_to_one` aceita dois perfis para a mesma pessoa:** isso é
  esperado na versão atual. A chave estrangeira não tem uma restrição
  `UNIQUE`, portanto esse exemplo ainda não implementa 1:1 de verdade.
- **A opção para escolher uma ficha não aparece:** confirme que já existe pelo
  menos uma ficha desse tipo no banco. A lista mostra apenas fichas salvas.
- **Alterações no model não aparecem no banco existente:** o schema usa
  `CREATE TABLE IF NOT EXISTS`, não um sistema de migrações. Faça uma migração
  ou reinicialize conscientemente o banco de desenvolvimento.

## 11. Os testes e o que o projeto ainda não faz

Os testes ficam em três lugares. `core/tests/generation.rs` chama diretamente
as funções geradoras e compara os textos ou erros esperados. `tests/lib.rs`
declara models de teste e confirma que as macros os registraram. Já
`docker/test_app.py` usa um arquivo SQLite temporário para testar inserções,
opções relacionais e consultas; um dos testes inicia um servidor HTTP local e
envia pedidos reais. Os arquivos temporários são removidos ao terminar cada
teste.

- O servidor é uma demonstração para aprender e testar, não está pronto para
  ser colocado na internet para uso real. Por exemplo, não verifica quem está
  usando o sistema nem limita quem pode ver ou alterar dados.
- As rotas que listam dados devolvem todas as fichas de uma vez. Em sistemas
  grandes, seria melhor dividi-las em páginas menores.
- É possível criar e consultar dados, mas ainda não há funções para editar ou
  apagar fichas.
- O código Rust não abre o banco para transformar automaticamente cada linha
  em uma struct Rust.
- O Djanho não atualiza automaticamente tabelas que já existam.
- As relações 1:N e N:N descritas acima são suportadas. A anotação atual não
  oferece uma regra `UNIQUE` para garantir 1:1.
- Declarar uma relação N:N ajuda a criar o formulário e a tabela de ligação,
  mas não preenche automaticamente uma lista dentro de uma struct Rust.
- Os nomes das structs e campos aparecem nos nomes das tabelas e nos endereços
  do servidor. Prefira nomes simples e sem espaços ou símbolos especiais.

## 12. Licença

O projeto é distribuído sob a licença MIT. Em resumo, ela permite usar,
modificar e compartilhar o código, desde que o aviso da licença acompanhe as
cópias. Consulte `LICENSE` para ler o texto completo.

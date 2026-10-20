"""Servidor HTTP simples que grava os formulários gerados em um banco SQLite.

As rotas de models são criadas a partir das tabelas do schema. O servidor
também carrega as opções dos campos relacionais e salva os vínculos N:N.
"""

import html
import json
import re
import sqlite3
from collections.abc import Mapping
from contextlib import closing
from http.server import BaseHTTPRequestHandler, HTTPServer
from urllib.parse import parse_qs, unquote, urlsplit

DB = "/data/db.sqlite"


def check_table_exists(table: str) -> bool:
    """Confere no catálogo do SQLite se uma tabela existe."""
    with closing(sqlite3.connect(DB)) as con:
        row = con.execute(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name=? COLLATE NOCASE",
            (table,),
        ).fetchone()
    return row is not None


def quote_identifier(identifier: str) -> str:
    """Protege um nome de tabela ou coluna usado dentro de SQL."""
    return '"' + identifier.replace('"', '""') + '"'


def get_columns(table: str, connection: sqlite3.Connection) -> list[tuple]:
    """Lê os nomes e tipos das colunas sem abrir outra conexão."""
    return connection.execute(
        f"PRAGMA table_info({quote_identifier(table)})"
    ).fetchall()


def build_query(
    table: str,
    data: Mapping[str, list[str]],
    connection: sqlite3.Connection | None = None,
) -> tuple[str, list]:
    """Monta um INSERT e seus valores usando parâmetros SQL.

    Valores do formulário nunca são concatenados ao SQL. Os nomes das colunas
    vêm do schema e são delimitados como identificadores do SQLite.
    """
    if connection is None:
        with closing(sqlite3.connect(DB)) as con:
            cols = get_columns(table, con)
    else:
        cols = get_columns(table, connection)

    names: list[str] = []
    values: list = []
    for _, name, ctype, notnull, _, pk in cols:
        # O banco preenche a chave primária; só inserimos as outras colunas.
        if pk:  # id autoincrementado fica de fora
            continue
        ctype = ctype.upper()
        raw = data.get(name)  # list[str] | None

        if "BOOL" in ctype:
            # Checkbox desmarcado não é enviado. Em campo obrigatório isso
            # significa falso; em Option<bool>, campo vazio significa None.
            if raw is None or raw[-1] == "":
                value = 0 if notnull else None
            elif raw[-1] in ("1", "true", "True"):
                value = 1
            elif raw[-1] in ("0", "false", "False"):
                value = 0
            else:
                raise ValueError(f"valor booleano inválido: {name}")
        elif raw is None or raw[-1] == "":
            if notnull:
                raise ValueError(f"campo obrigatório ausente: {name}")
            value = None
        elif "INT" in ctype:
            # Chaves estrangeiras (1:N) chegam aqui: só o id é enviado.
            value = int(raw[-1])
        elif any(t in ctype for t in ("REAL", "FLOA", "DOUB")):
            value = float(raw[-1])
        else:
            value = raw[-1]

        names.append(quote_identifier(name))
        values.append(value)

    if names:
        placeholders = ", ".join("?" * len(names))
        query = (
            f"INSERT INTO {quote_identifier(table)} ({', '.join(names)}) "
            f"VALUES ({placeholders})"
        )
    else:
        query = f"INSERT INTO {quote_identifier(table)} DEFAULT VALUES"
    return query, values


def get_link_foreign_keys(
    table: str, connection: sqlite3.Connection
) -> list[tuple[str, str]]:
    """Confirma que a tabela é uma associação N:N e retorna suas duas FKs."""
    columns = get_columns(table, connection)
    primary_key_columns = [row[1] for row in columns if row[5]]
    foreign_keys = connection.execute(
        f"PRAGMA foreign_key_list({quote_identifier(table)})"
    ).fetchall()
    if len(foreign_keys) != 2 or len(primary_key_columns) != 2:
        raise ValueError(f"tabela '{table}' não é uma tabela de associação N:N")

    references = [(row[3], row[2]) for row in foreign_keys]
    if set(primary_key_columns) != {column for column, _ in references}:
        raise ValueError(f"tabela '{table}' não possui chave primária composta válida")
    return references


def insert_record(
    table: str, data: Mapping[str, list[str]]
) -> int:
    """Insere um registro e, na mesma transação, os vínculos N:N selecionados."""
    with closing(sqlite3.connect(DB)) as connection:
        # SQLite mantém FKs desligadas por padrão; ativamos em cada conexão.
        connection.execute("PRAGMA foreign_keys = ON")
        query, values = build_query(table, data, connection)
        cursor = connection.execute(query, values)
        record_id = cursor.lastrowid

        for key, related_ids in data.items():
            # Os campos especiais são adicionados pelos controles N:N gerados.
            if not key.startswith("__relation__"):
                continue
            relation_table = key.removeprefix("__relation__")
            references = get_link_foreign_keys(relation_table, connection)
            source_columns = [
                column for column, target_table in references if target_table == table
            ]
            if not source_columns:
                raise ValueError(
                    f"tabela '{relation_table}' não referencia o model '{table}'"
                )

            if len(source_columns) == 2:
                # Em autorrelações, as duas FKs apontam para a mesma tabela.
                source_column = "source_id" if "source_id" in source_columns else source_columns[0]
                target_column = next(
                    column for column in source_columns if column != source_column
                )
            else:
                source_column = source_columns[0]
                target_column, _ = next(
                    (column, model)
                    for column, model in references
                    if column != source_column
                )

            statement = (
                f"INSERT INTO {quote_identifier(relation_table)} "
                f"({quote_identifier(source_column)}, {quote_identifier(target_column)}) "
                "VALUES (?, ?) "
                f"ON CONFLICT ({quote_identifier(source_column)}, "
                f"{quote_identifier(target_column)}) DO NOTHING"
            )
            for raw_id in related_ids:
                # Valores vazios não representam vínculo; ignorar em vez de falhar.
                if raw_id == "":
                    continue
                try:
                    related_id = int(raw_id)
                except ValueError as error:
                    raise ValueError("id relacionado inválido") from error
                if related_id <= 0:
                    raise ValueError("id relacionado inválido")
                if len(source_columns) == 2:
                    if record_id == related_id:
                        raise ValueError("um registro não pode se relacionar consigo mesmo")
                    first_id, second_id = sorted((record_id, related_id))
                    connection.execute(statement, (first_id, second_id))
                else:
                    connection.execute(statement, (record_id, related_id))

        connection.commit()
        return record_id


def insert_relationship(table: str, data: Mapping[str, list[str]]) -> None:
    """Associa dois registros existentes por uma tabela de ligação N:N."""
    with closing(sqlite3.connect(DB)) as connection:
        connection.execute("PRAGMA foreign_keys = ON")
        references = get_link_foreign_keys(table, connection)
        related_ids: dict[str, int] = {}

        for column, _ in references:
            raw_values = data.get(column)
            if not raw_values or raw_values[-1] == "":
                raise ValueError(f"registro relacionado ausente: {column}")
            try:
                related_id = int(raw_values[-1])
            except ValueError as error:
                raise ValueError(f"id relacionado inválido: {column}") from error
            if related_id <= 0:
                raise ValueError(f"id relacionado inválido: {column}")
            related_ids[column] = related_id

        if references[0][1] == references[1][1]:
            if not {"source_id", "target_id"}.issubset(related_ids):
                raise ValueError(f"tabela de autorrelação '{table}' inválida")
            if related_ids["source_id"] == related_ids["target_id"]:
                raise ValueError("um registro não pode se relacionar consigo mesmo")
            columns = ["source_id", "target_id"]
            values = sorted(related_ids[column] for column in columns)
        else:
            columns = [column for column, _ in references]
            values = [related_ids[column] for column in columns]

        quoted_columns = [quote_identifier(column) for column in columns]
        connection.execute(
            f"INSERT INTO {quote_identifier(table)} ({', '.join(quoted_columns)}) "
            f"VALUES (?, ?) ON CONFLICT DO NOTHING",
            values,
        )
        connection.commit()


def fetch_rows(table: str) -> list[dict]:
    """Retorna todas as linhas de uma tabela como uma lista de dicionários."""
    with closing(sqlite3.connect(DB)) as con:
        cur = con.execute(f"SELECT * FROM {quote_identifier(table)}")
        names = [d[0] for d in cur.description]
        return [dict(zip(names, row)) for row in cur.fetchall()]


def populate_form_options(form: str) -> str:
    """Preenche os selects relacionais no HTML antes de enviar ao navegador.

    O value de cada opção é sempre o id do registro, que é o único dado
    enviado no POST. O texto serve só para o usuário reconhecer o registro
    (e é o que a pesquisa do dropdown filtra).
    """
    pattern = re.compile(
        r'(<select\b(?=[^>]*\bdata-djanho-target="([^"]+)")[^>]*>)'
        r"(.*?)</select>",
        re.DOTALL,
    )

    def populate(match: re.Match) -> str:
        # O atributo data-djanho-target identifica a tabela de onde vêm opções.
        target = match.group(2)
        if not check_table_exists(target):
            raise ValueError(f"model relacionado '{target}' não existe no schema")

        with closing(sqlite3.connect(DB)) as connection:
            columns = get_columns(target, connection)
            column_names = {row[1] for row in columns}
            if "id" not in column_names:
                raise ValueError(f"model relacionado '{target}' não possui coluna id")

            # A coluna descritiva é opcional: sem ela, o texto é só o id.
            display_column = next(
                (name for name in ("name", "title", "label") if name in column_names),
                None,
            )
            if display_column is None:
                rows = connection.execute(
                    f"SELECT id, NULL FROM {quote_identifier(target)} ORDER BY id"
                ).fetchall()
            else:
                rows = connection.execute(
                    f"SELECT id, {quote_identifier(display_column)} "
                    f"FROM {quote_identifier(target)} ORDER BY id"
                ).fetchall()

        options = "".join(
            f'<option value="{html.escape(str(record_id), quote=True)}">'
            f"{html.escape(str(record_id) if label is None else f'{record_id} — {label}')}"
            "</option>"
            for record_id, label in rows
        )
        return f"{match.group(1)}{match.group(3)}{options}</select>"

    return pattern.sub(populate, form)


def fetch_related(table: str, record_id: int, relation_table: str) -> list[dict]:
    """Busca os modelos associados a um registro por uma tabela N:N."""
    with closing(sqlite3.connect(DB)) as connection:
        references = get_link_foreign_keys(relation_table, connection)
        source_columns = [
            column for column, target_table in references if target_table == table
        ]
        if not source_columns:
            raise ValueError(f"'{relation_table}' não relaciona o model '{table}'")

        if len(source_columns) == 2:
            # Se os dois lados apontam para o mesmo model, procura o id nas
            # duas colunas para tratar a associação de forma simétrica.
            first, second = source_columns
            query = (
                f"SELECT target.* FROM {quote_identifier(table)} AS target "
                f"JOIN {quote_identifier(relation_table)} AS relation "
                f"ON relation.{quote_identifier(second)} = target.id "
                f"WHERE relation.{quote_identifier(first)} = ? "
                "UNION "
                f"SELECT target.* FROM {quote_identifier(table)} AS target "
                f"JOIN {quote_identifier(relation_table)} AS relation "
                f"ON relation.{quote_identifier(first)} = target.id "
                f"WHERE relation.{quote_identifier(second)} = ?"
            )
            rows = connection.execute(query, (record_id, record_id)).fetchall()
            target_table = table
        else:
            source_column = source_columns[0]
            target_column, target_table = next(
                (column, model)
                for column, model in references
                if column != source_column
            )
            query = (
                f"SELECT target.* FROM {quote_identifier(target_table)} AS target "
                f"JOIN {quote_identifier(relation_table)} AS relation "
                f"ON target.id = relation.{quote_identifier(target_column)} "
                f"WHERE relation.{quote_identifier(source_column)} = ?"
            )
            rows = connection.execute(query, (record_id,)).fetchall()

        names = [row[1] for row in get_columns(target_table, connection)]
        return [dict(zip(names, row)) for row in rows]


def fetch_children(
    parent_table: str, parent_id: int, child_table: str, foreign_key: str
) -> list[dict]:
    """Busca os registros do lado "muitos" de uma relação 1:N."""
    with closing(sqlite3.connect(DB)) as connection:
        columns = get_columns(child_table, connection)
        matching = [
            row
            for row in connection.execute(
                f"PRAGMA foreign_key_list({quote_identifier(child_table)})"
            )
            if row[2] == parent_table and row[3] == foreign_key
        ]
        if not matching or foreign_key not in {column[1] for column in columns}:
            raise ValueError(
                f"'{child_table}.{foreign_key}' não referencia '{parent_table}'"
            )
        rows = connection.execute(
            f"SELECT * FROM {quote_identifier(child_table)} "
            f"WHERE {quote_identifier(foreign_key)} = ?",
            (parent_id,),
        ).fetchall()
        names = [row[1] for row in columns]
        return [dict(zip(names, row)) for row in rows]


def init_db():
    """Cria ou atualiza as tabelas executando o schema gerado pelo Rust."""
    with closing(sqlite3.connect(DB)) as con:
        con.executescript(open("schema.sql").read())


class Handler(BaseHTTPRequestHandler):
    def _status(self, code: int):
        """Envia uma resposta sem conteúdo, usada para erros e rotas inválidas."""
        self.send_response(code)
        self.send_header("Content-Length", "0")
        self.end_headers()

    def _send(self, code: int, content_type: str, body: bytes):
        """Envia uma resposta com tipo de conteúdo e corpo explícitos."""
        self.send_response(code)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        """Atende o formulário, consultas de tabelas e rotas de relações."""
        path = unquote(urlsplit(self.path).path)
        if path == "/form":
            with open("form.html", encoding="utf-8") as form_file:
                body = populate_form_options(form_file.read()).encode()
            self._send(200, "text/html; charset=utf-8", body)
            return

        parts = [part for part in path.split("/") if part]
        if len(parts) == 4 and parts[2] == "related":
            table, raw_id, _, relation_table = parts
            try:
                rows = fetch_related(table, int(raw_id), relation_table)
            except (ValueError, sqlite3.Error) as error:
                print("erro:", error, flush=True)
                self._status(404)
                return
            body = json.dumps(rows, ensure_ascii=False).encode()
            self._send(200, "application/json; charset=utf-8", body)
            return

        if len(parts) == 5 and parts[2] == "children":
            table, raw_id, _, child_table, foreign_key = parts
            try:
                rows = fetch_children(table, int(raw_id), child_table, foreign_key)
            except (ValueError, sqlite3.Error) as error:
                print("erro:", error, flush=True)
                self._status(404)
                return
            body = json.dumps(rows, ensure_ascii=False).encode()
            self._send(200, "application/json; charset=utf-8", body)
            return

        if len(parts) != 1:
            self._status(404)
            return

        table = parts[0]
        if not check_table_exists(table):
            self._status(404)
            return

        body = json.dumps(fetch_rows(table), ensure_ascii=False).encode()
        self._send(200, "application/json; charset=utf-8", body)

    def do_POST(self):
        """Salva um registro ou um vínculo N:N e redireciona ao formulário."""
        size = int(self.headers.get("Content-Length", 0))
        data = parse_qs(self.rfile.read(size).decode())
        parts = [part for part in unquote(urlsplit(self.path).path).split("/") if part]

        if len(parts) == 2 and parts[0] == "relationships":
            try:
                insert_relationship(parts[1], data)
            except (ValueError, sqlite3.Error) as error:
                print("erro:", error, flush=True)
                self._status(400)
                return

            self.send_response(303)
            self.send_header("Location", "/form")
            self.send_header("Content-Length", "0")
            self.end_headers()
            return

        if len(parts) != 1:
            self._status(404)
            return
        table = parts[0]

        if not check_table_exists(table):
            self._status(404)
            return

        try:
            insert_record(table, data)
        except (ValueError, sqlite3.IntegrityError) as e:
            print("erro:", e, flush=True)
            self._status(400)
            return

        self.send_response(303)
        self.send_header("Location", "/form")
        self.send_header("Content-Length", "0")
        self.end_headers()


if __name__ == "__main__":
    init_db()
    print("Djanho ORM funcionando :)\nMensagens aparecerão aqui.", flush=True)
    HTTPServer(("0.0.0.0", 8000), Handler).serve_forever()
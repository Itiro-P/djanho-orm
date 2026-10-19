import sqlite3
from collections.abc import Mapping
from contextlib import closing
from http.server import BaseHTTPRequestHandler, HTTPServer
from urllib.parse import parse_qs
from string import capwords

DB = "/data/db.sqlite"


def extract_path(path: str) -> str:
    return path.removeprefix("/")

def check_table_exists(table: str) -> bool:
    with closing(sqlite3.connect(DB)) as con:
        row = con.execute(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name=? COLLATE NOCASE",
            (table,),
        ).fetchone()
    return row is not None


def build_query(table: str, data: Mapping[str, list[str]]) -> tuple[str, list]:
    # 'table' já foi validada contra sqlite_master, então é seguro interpolar.
    with closing(sqlite3.connect(DB)) as con:
        cols = con.execute(f'PRAGMA table_info("{table}")').fetchall()

    names: list[str] = []
    values: list = []
    for _, name, ctype, notnull, _, pk in cols:
        if pk:  # id autoincrementado fica de fora
            continue
        ctype = ctype.upper()
        raw = data.get(name)  # list[str] | None

        if "BOOL" in ctype:
            value = 1 if raw is not None else 0  # checkbox ausente = falso
        elif raw is None or raw[-1] == "":
            if notnull:
                raise ValueError(f"campo obrigatório ausente: {name}")
            value = None
        elif "INT" in ctype:
            value = int(raw[-1])
        elif any(t in ctype for t in ("REAL", "FLOA", "DOUB")):
            value = float(raw[-1])
        else:
            value = raw[-1]

        names.append(f'"{name}"')
        values.append(value)

    placeholders = ", ".join("?" * len(names))
    query = f'INSERT INTO "{table}" ({", ".join(names)}) VALUES ({placeholders})'
    return query, values


def init_db():
    with closing(sqlite3.connect(DB)) as con:
        con.executescript(open("schema.sql").read())


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        with open("form.html", "rb") as f:
            body = f.read()
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _status(self, code: int):
        self.send_response(code)
        self.send_header("Content-Length", "0")
        self.end_headers()

    def do_POST(self):
        size = int(self.headers.get("Content-Length", 0))
        data = parse_qs(self.rfile.read(size).decode())
        table = extract_path(self.path)
        print(self.path, data, flush=True)

        if not check_table_exists(table):
            self._status(404)
            return

        try:
            query, values = build_query(table, data)
            with closing(sqlite3.connect(DB)) as con:
                con.execute("PRAGMA foreign_keys = ON")
                con.execute(query, values)
                con.commit()
        except (ValueError, sqlite3.IntegrityError) as e:
            print("erro:", e, flush=True)
            self._status(400)
            return

        self.send_response(303)
        self.send_header("Location", "/")
        self.end_headers()


if __name__ == "__main__":
    init_db()
    HTTPServer(("0.0.0.0", 8000), Handler).serve_forever()
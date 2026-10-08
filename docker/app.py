import sqlite3
from http.server import BaseHTTPRequestHandler, HTTPServer
from urllib.parse import parse_qs

DB = "/data/db.sqlite"

def init_db():
    con = sqlite3.connect(DB)
    con.executescript(open("schema.sql").read())
    con.close()

class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        with open("form.html", "rb") as f:
            corpo = f.read()
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(corpo)))
        self.end_headers()
        self.wfile.write(corpo)
    def do_POST(self):
        # 1. ler Content-Length e o corpo (self.rfile.read)
        # 2. parse_qs(corpo.decode()) -> dict de listas
        # 3. INSERT com placeholders (?), nunca f-string
        # 4. responder 303 com Location: /
        pass

if __name__ == "__main__":
    init_db()
    HTTPServer(("0.0.0.0", 8000), Handler).serve_forever()
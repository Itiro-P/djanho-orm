"""Testes do backend HTTP e SQLite usando bancos temporários."""

import json
import os
import sqlite3
import tempfile
import threading
import unittest
from contextlib import closing
from http.client import HTTPConnection
from http.server import HTTPServer
from urllib.parse import urlencode

import app


class RelationshipBackendTests(unittest.TestCase):
    """Cada teste começa com um schema isolado para não compartilhar dados."""

    def setUp(self):
        """Cria tabelas de exemplo e aponta o servidor para um arquivo temporário."""
        handle, self.database = tempfile.mkstemp(suffix=".sqlite")
        os.close(handle)
        app.DB = self.database
        with closing(sqlite3.connect(app.DB)) as connection:
            connection.executescript(
                """
                PRAGMA foreign_keys = ON;
                CREATE TABLE Teacher (id INTEGER PRIMARY KEY, name TEXT NOT NULL);
                CREATE TABLE Class (
                    id INTEGER PRIMARY KEY,
                    teacher_id INTEGER REFERENCES Teacher(id)
                );
                CREATE TABLE BooleanValues (
                    id INTEGER PRIMARY KEY,
                    mandatory BOOLEAN NOT NULL CHECK (mandatory IN (0, 1)),
                    optional BOOLEAN CHECK (optional IN (0, 1))
                );
                CREATE TABLE Student (id INTEGER PRIMARY KEY, name TEXT NOT NULL);
                CREATE TABLE Course (id INTEGER PRIMARY KEY, title TEXT NOT NULL);
                CREATE TABLE LinkOnlyModel (id INTEGER PRIMARY KEY);
                CREATE TABLE LinkOnlyModel_Course_Link (
                    linkonlymodel_id INTEGER NOT NULL REFERENCES LinkOnlyModel(id),
                    course_id INTEGER NOT NULL REFERENCES Course(id),
                    PRIMARY KEY (linkonlymodel_id, course_id)
                );
                CREATE TABLE Student_Course_Link (
                    student_id INTEGER NOT NULL REFERENCES Student(id),
                    course_id INTEGER NOT NULL REFERENCES Course(id),
                    PRIMARY KEY (student_id, course_id)
                );
                CREATE TABLE Employee (
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL
                );
                CREATE TABLE Employee_Employee_Link (
                    source_id INTEGER NOT NULL REFERENCES Employee(id),
                    target_id INTEGER NOT NULL REFERENCES Employee(id),
                    PRIMARY KEY (source_id, target_id)
                );
                """
            )
            connection.commit()

    def tearDown(self):
        """Apaga o banco temporário ao terminar o teste."""
        os.unlink(self.database)

    def test_create_parent_and_n_to_n_links(self):
        """Confirma que uma seção separada associa dois registros existentes."""
        with closing(sqlite3.connect(app.DB)) as connection:
            student_id = connection.execute(
                "INSERT INTO Student (name) VALUES (?)", ("Ana",)
            ).lastrowid
            course_id = connection.execute(
                "INSERT INTO Course (title) VALUES (?)", ("Rust",)
            ).lastrowid
            connection.commit()

        app.insert_relationship(
            "Student_Course_Link",
            {
                "student_id": [str(student_id)],
                "course_id": [str(course_id)],
            },
        )

        related = app.fetch_related("Student", student_id, "Student_Course_Link")
        self.assertEqual([row["title"] for row in related], ["Rust"])

    def test_relationship_section_rejects_missing_or_same_self_record(self):
        """Uma seção N:N exige dois registros válidos e distintos."""
        with self.assertRaises(ValueError):
            app.insert_relationship("Student_Course_Link", {"student_id": ["1"]})

        with closing(sqlite3.connect(app.DB)) as connection:
            employee_ids = [
                connection.execute(
                    "INSERT INTO Employee (name) VALUES (?)", (name,)
                ).lastrowid
                for name in ("Ana", "Bia")
            ]
            connection.commit()

        app.insert_relationship(
            "Employee_Employee_Link",
            {
                "source_id": [str(employee_ids[1])],
                "target_id": [str(employee_ids[0])],
            },
        )
        related = app.fetch_related(
            "Employee", employee_ids[0], "Employee_Employee_Link"
        )
        self.assertEqual([row["name"] for row in related], ["Bia"])

        with self.assertRaises(ValueError):
            app.insert_relationship(
                "Employee_Employee_Link",
                {
                    "source_id": [str(employee_ids[0])],
                    "target_id": [str(employee_ids[0])],
                },
            )

    def test_http_post_and_relationship_routes(self):
        """Exercita cadastro, associação N:N e consultas pela interface HTTP."""
        with closing(sqlite3.connect(app.DB)) as connection:
            course_id = connection.execute(
                "INSERT INTO Course (title) VALUES (?)", ("Rust",)
            ).lastrowid
            connection.commit()

        student_id = app.insert_record(
            "Student",
            {
                "name": ["Ana"],
                "__relation__Student_Course_Link": [str(course_id)],
            },
        )

        related = app.fetch_related("Student", student_id, "Student_Course_Link")
        self.assertEqual([row["title"] for row in related], ["Rust"])

    def test_relationship_only_model_uses_default_values_insert(self):
        """Confirma que um model sem campos pode ser criado só para ter relações."""
        with closing(sqlite3.connect(app.DB)) as connection:
            course_id = connection.execute(
                "INSERT INTO Course (title) VALUES (?)", ("Rust",)
            ).lastrowid
            connection.commit()

        link_only_id = app.insert_record(
            "LinkOnlyModel",
            {
                "__relation__LinkOnlyModel_Course_Link": [str(course_id)],
            },
        )
        related = app.fetch_related(
            "LinkOnlyModel", link_only_id, "LinkOnlyModel_Course_Link"
        )
        self.assertEqual(related[0]["title"], "Rust")

    def test_create_link_from_inverse_form_side(self):
        """Confirma que o vínculo também funciona pelo outro lado da relação."""
        with closing(sqlite3.connect(app.DB)) as connection:
            student_id = connection.execute(
                "INSERT INTO Student (name) VALUES (?)", ("Ana",)
            ).lastrowid
            connection.commit()

        course_id = app.insert_record(
            "Course",
            {
                "title": ["Rust"],
                "__relation__Student_Course_Link": [str(student_id)],
            },
        )

        related = app.fetch_related("Course", course_id, "Student_Course_Link")
        self.assertEqual([row["name"] for row in related], ["Ana"])

    def test_self_relation_is_symmetric_for_queries(self):
        """Confirma que a consulta encontra uma autorrelação em qualquer direção."""
        with closing(sqlite3.connect(app.DB)) as connection:
            employee_ids = [
                connection.execute(
                    "INSERT INTO Employee (name) VALUES (?)", (name,)
                ).lastrowid
                for name in ("Ana", "Bia")
            ]
            connection.commit()

        app.insert_record(
            "Employee",
            {
                "name": ["Caio"],
                "__relation__Employee_Employee_Link": [str(employee_ids[0])],
            },
        )
        app.insert_record(
            "Employee",
            {
                "name": ["Davi"],
                "__relation__Employee_Employee_Link": [str(employee_ids[1])],
            },
        )
        app.insert_record(
            "Employee",
            {
                "name": ["Ana linked"],
                "__relation__Employee_Employee_Link": [str(employee_ids[0])],
            },
        )

        # Cria um par direto para verificar a consulta simétrica.
        with closing(sqlite3.connect(app.DB)) as connection:
            connection.execute(
                "INSERT OR IGNORE INTO Employee_Employee_Link VALUES (?, ?)",
                (employee_ids[0], employee_ids[1]),
            )
            connection.commit()
        related = app.fetch_related(
            "Employee", employee_ids[1], "Employee_Employee_Link"
        )
        self.assertIn("Ana", [row["name"] for row in related])

    def test_nullable_one_to_many_foreign_key_accepts_empty_value(self):
        """Um campo FK opcional vazio deve ser gravado como NULL."""
        app.insert_record("Class", {"teacher_id": [""]})
        self.assertEqual(app.fetch_rows("Class")[0]["teacher_id"], None)

    def test_boolean_form_values_distinguish_false_from_absent_optional(self):
        """Distingue false de None para manter os três estados de Option<bool>."""
        app.insert_record("BooleanValues", {"optional": ["0"]})
        app.insert_record("BooleanValues", {"mandatory": ["1"]})

        rows = app.fetch_rows("BooleanValues")
        self.assertEqual(rows[0]["mandatory"], 0)
        self.assertEqual(rows[0]["optional"], 0)
        self.assertEqual(rows[1]["mandatory"], 1)
        self.assertIsNone(rows[1]["optional"])

    def test_form_options_are_loaded_and_html_escaped(self):
        """As opções vêm do banco e textos são escapados para evitar HTML injetado."""
        with closing(sqlite3.connect(app.DB)) as connection:
            connection.execute("INSERT INTO Course (title) VALUES (?)", ("<Rust>",))
            connection.commit()

        form = (
            '<select name="course_id" data-djanho-target="Course">'
            "</select>"
        )
        rendered = app.populate_form_options(form)
        self.assertIn("&lt;Rust&gt;", rendered)
        self.assertIn('<option value="1">', rendered)

    def test_unrelated_table_cannot_be_used_as_join_table(self):
        """Uma tabela sem duas FKs e chave composta não pode receber vínculos."""
        with self.assertRaises(ValueError):
            app.insert_record(
                "Student",
                {
                    "name": ["Ana"],
                    "__relation__Course": ["1"],
                },
            )

    def test_one_to_many_children_query_checks_foreign_key(self):
        """A rota 1:N só aceita a FK que realmente aponta para o model pai."""
        with closing(sqlite3.connect(app.DB)) as connection:
            teacher_id = connection.execute(
                "INSERT INTO Teacher (name) VALUES (?)", ("Marta",)
            ).lastrowid
            connection.execute(
                "INSERT INTO Class (teacher_id) VALUES (?)", (teacher_id,)
            )
            connection.commit()

        children = app.fetch_children("Teacher", teacher_id, "Class", "teacher_id")
        self.assertEqual(len(children), 1)

        with self.assertRaises(ValueError):
            app.fetch_children("Teacher", teacher_id, "Class", "id")

    def test_http_post_and_relationship_routes(self):
        """Exercita POST e rotas de consulta pela interface HTTP real."""
        server = HTTPServer(("127.0.0.1", 0), app.Handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        client = HTTPConnection("127.0.0.1", server.server_port)
        try:
            client.request(
                "POST",
                "/Course",
                urlencode({"title": "Rust"}),
                {"Content-Type": "application/x-www-form-urlencoded"},
            )
            self.assertEqual(client.getresponse().status, 303)

            with closing(sqlite3.connect(app.DB)) as connection:
                course_id = connection.execute("SELECT id FROM Course").fetchone()[0]

            client.request(
                "POST",
                "/Student",
                urlencode({"name": "Ana"}),
                {"Content-Type": "application/x-www-form-urlencoded"},
            )
            self.assertEqual(client.getresponse().status, 303)

            with closing(sqlite3.connect(app.DB)) as connection:
                student_id = connection.execute("SELECT id FROM Student").fetchone()[0]

            client.request(
                "POST",
                "/relationships/Student_Course_Link",
                urlencode(
                    {
                        "student_id": str(student_id),
                        "course_id": str(course_id),
                    }
                ),
                {"Content-Type": "application/x-www-form-urlencoded"},
            )
            self.assertEqual(client.getresponse().status, 303)

            client.request(
                "GET", f"/Student/{student_id}/related/Student_Course_Link"
            )
            response = client.getresponse()
            self.assertEqual(response.status, 200)
            self.assertEqual(json.loads(response.read())[0]["title"], "Rust")

            client.request(
                "POST", "/Teacher", urlencode({"name": "Marta"}),
                {"Content-Type": "application/x-www-form-urlencoded"},
            )
            self.assertEqual(client.getresponse().status, 303)
            with closing(sqlite3.connect(app.DB)) as connection:
                teacher_id = connection.execute("SELECT id FROM Teacher").fetchone()[0]

            client.request(
                "POST",
                "/Class",
                urlencode({"teacher_id": str(teacher_id)}),
                {"Content-Type": "application/x-www-form-urlencoded"},
            )
            self.assertEqual(client.getresponse().status, 303)
            client.request(
                "GET", f"/Teacher/{teacher_id}/children/Class/teacher_id"
            )
            response = client.getresponse()
            self.assertEqual(response.status, 200)
            self.assertEqual(len(json.loads(response.read())), 1)
        finally:
            client.close()
            server.shutdown()
            server.server_close()
            thread.join()


if __name__ == "__main__":
    unittest.main()

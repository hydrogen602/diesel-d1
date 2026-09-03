DROP TABLE IF EXISTS js_limits;
DROP TABLE IF EXISTS posts;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS notes;
DROP TABLE IF EXISTS typed;
CREATE TABLE notes (
  id INTEGER PRIMARY KEY,
  label TEXT NOT NULL DEFAULT 'untitled'
) STRICT;
CREATE TABLE users (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  created_at TEXT NOT NULL
) STRICT;
CREATE TABLE posts (
  id INTEGER PRIMARY KEY,
  title TEXT NOT NULL,
  body TEXT,
  user_id INTEGER NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users (id)
) STRICT;
CREATE TABLE typed (
  id INTEGER PRIMARY KEY,
  score REAL NOT NULL,
  payload BLOB
) STRICT;
INSERT INTO users (id, name, created_at)
VALUES (1, 'John Doe', '2021-01-01'),
  (2, 'Jane Smith', '2021-01-02'),
  (3, 'Jim Beam', '2021-01-03'),
  (4, 'Jane Doe', '2021-01-04');
INSERT INTO posts (id, title, body, user_id)
VALUES (1, 'Hello World', 'This is a test post', 1),
  (
    2,
    'Another Post',
    'This is another test post',
    1
  ),
  (3, 'Post #3', 'Lots of words', 2),
  (4, 'Post #4', 'Even more words', 2),
  (5, 'Post #5', 'Imagine a post here', 2);
INSERT INTO typed (id, score, payload)
VALUES (1, 1.5, X'01'),
  (2, 2.5, X'02'),
  (3, 3.5, NULL),
  (4, 4.5, X'0102');
CREATE TABLE js_limits (
  id INTEGER PRIMARY KEY,
  int_val INTEGER NOT NULL,
  real_val REAL NOT NULL
) STRICT;
INSERT INTO js_limits (id, int_val, real_val)
VALUES (1, 9007199254740991, 1.0e20),
  (2, -9007199254740991, -1.0e20),
  (3, 9007199254740993, 9007199254740992),
  (4, -9007199254740993, -9007199254740992);
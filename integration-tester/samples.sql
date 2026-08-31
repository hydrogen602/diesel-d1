DROP TABLE IF EXISTS posts;
DROP TABLE IF EXISTS users;
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
INSERT INTO users (id, name, created_at)
VALUES (1, 'John Doe', '2021-01-01'),
  (2, 'Jane Smith', '2021-01-02');
INSERT INTO posts (id, title, body, user_id)
VALUES (1, 'Hello World', 'This is a test post', 1),
  (
    2,
    'Another Post',
    'This is another test post',
    1
  );
//! See if there is a way to conditionally compile this

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use worker::*;

use crate::D1Connection;

mod sample_schema {
    diesel::table! {
        users (id) {
            id -> Integer,
            name -> Text,
            created_at -> Text,
        }
    }

    diesel::table! {
        posts (id) {
            id -> Integer,
            title -> Text,
            body -> Nullable<Text>,
            user_id -> Integer,
        }
    }

    diesel::joinable!(posts -> users (user_id));
    diesel::allow_tables_to_appear_in_same_query!(posts, users);
}

#[derive(Queryable, Debug)]
struct User {
    id: i32,
    name: String,
    created_at: String,
}

#[derive(Queryable, Debug)]
struct Post {
    id: i32,
    title: String,
    body: Option<String>,
    user_id: i32,
}

pub async fn test_users(env: &Env) {
    let mut d1 = D1Connection::new(&env, "diesel_d1_test").unwrap();

    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .filter(sample_schema::users::id.lt(3));

    let rows: Vec<User> = query.load(&mut d1).await.unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].id, 1);
    assert_eq!(rows[0].name, "John Doe");
    assert_eq!(rows[0].created_at, "2021-01-01");
    assert_eq!(rows[1].id, 2);
    assert_eq!(rows[1].name, "Jane Smith");
    assert_eq!(rows[1].created_at, "2021-01-02");
}

pub async fn test_posts(env: &Env) {
    let mut d1 = D1Connection::new(&env, "diesel_d1_test").unwrap();

    let query = sample_schema::posts::table.select(sample_schema::posts::all_columns);

    let rows: Vec<Post> = query.load(&mut d1).await.unwrap();

    assert_eq!(rows.len(), 5);

    assert_eq!(rows[0].id, 1);
    assert_eq!(rows[0].title, "Hello World");
    assert_eq!(rows[0].body, Some("This is a test post".to_string()));
    assert_eq!(rows[0].user_id, 1);
    assert_eq!(rows[1].id, 2);
    assert_eq!(rows[1].title, "Another Post");
    assert_eq!(rows[1].body, Some("This is another test post".to_string()));
    assert_eq!(rows[1].user_id, 1);
    assert_eq!(rows[2].id, 3);
    assert_eq!(rows[2].title, "Post #3");
    assert_eq!(rows[2].body, Some("Lots of words".to_string()));
    assert_eq!(rows[2].user_id, 2);
    assert_eq!(rows[3].id, 4);
    assert_eq!(rows[3].title, "Post #4");
    assert_eq!(rows[3].body, Some("Even more words".to_string()));
    assert_eq!(rows[3].user_id, 2);
    assert_eq!(rows[4].id, 5);
    assert_eq!(rows[4].title, "Post #5");
    assert_eq!(rows[4].body, Some("Imagine a post here".to_string()));
    assert_eq!(rows[4].user_id, 2);

    // foreign key tests - join

    let query = sample_schema::posts::table
        .inner_join(sample_schema::users::table)
        .select((
            sample_schema::posts::all_columns,
            sample_schema::users::name,
        ));

    let rows: Vec<(Post, String)> = query.load(&mut d1).await.unwrap();

    assert_eq!(rows.len(), 5);

    assert_eq!(rows[0].0.id, 1);
    assert_eq!(rows[0].0.title, "Hello World");
    assert_eq!(rows[0].0.body, Some("This is a test post".to_string()));
    assert_eq!(rows[0].0.user_id, 1);
    assert_eq!(rows[0].1, "John Doe");
    assert_eq!(rows[1].0.id, 2);
    assert_eq!(rows[1].0.title, "Another Post");
    assert_eq!(
        rows[1].0.body,
        Some("This is another test post".to_string())
    );
    assert_eq!(rows[1].0.user_id, 1);
    assert_eq!(rows[1].1, "John Doe");
    assert_eq!(rows[2].0.id, 3);
    assert_eq!(rows[2].0.title, "Post #3");
    assert_eq!(rows[2].0.body, Some("Lots of words".to_string()));
    assert_eq!(rows[2].0.user_id, 2);
    assert_eq!(rows[2].1, "Jane Smith");
    assert_eq!(rows[3].0.id, 4);
    assert_eq!(rows[3].0.title, "Post #4");
    assert_eq!(rows[3].0.body, Some("Even more words".to_string()));
    assert_eq!(rows[3].0.user_id, 2);
    assert_eq!(rows[3].1, "Jane Smith");
    assert_eq!(rows[4].0.id, 5);
    assert_eq!(rows[4].0.title, "Post #5");
    assert_eq!(rows[4].0.body, Some("Imagine a post here".to_string()));
    assert_eq!(rows[4].0.user_id, 2);
    assert_eq!(rows[4].1, "Jane Smith");
}

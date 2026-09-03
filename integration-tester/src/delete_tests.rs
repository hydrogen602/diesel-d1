use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_d1::SessionOptions;
use worker::*;

use crate::{D1_NAME, D1Connection};

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
}

#[derive(Queryable, Debug)]
struct User {
    id: i32,
    name: String,
    created_at: String,
}

#[derive(Queryable, Identifiable, Debug)]
#[diesel(table_name = sample_schema::posts)]
struct Post {
    id: i32,
    title: String,
    body: Option<String>,
    user_id: i32,
}

fn assert_user(row: &User, id: i32, name: &str, created_at: &str) {
    assert_eq!(row.id, id);
    assert_eq!(row.name, name);
    assert_eq!(row.created_at, created_at);
}

fn assert_post(row: &Post, id: i32, title: &str, body: Option<&str>, user_id: i32) {
    assert_eq!(row.id, id);
    assert_eq!(row.title, title);
    assert_eq!(row.body.as_deref(), body);
    assert_eq!(row.user_id, user_id);
}

async fn load_users(d1: &mut D1Connection) -> Vec<User> {
    sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .order(sample_schema::users::id)
        .load(d1)
        .await
        .unwrap()
}

async fn load_posts(d1: &mut D1Connection) -> Vec<Post> {
    sample_schema::posts::table
        .select(sample_schema::posts::all_columns)
        .order(sample_schema::posts::id)
        .load(d1)
        .await
        .unwrap()
}

pub async fn test_delete_all(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME, SessionOptions::default()).unwrap();

    // Test: Del1 - All rows
    let delete = diesel::delete(sample_schema::posts::table);

    // Test: DelE1 - Affected row count
    let count = delete.execute(&mut d1).await.unwrap();
    assert_eq!(count, 5);

    let posts = load_posts(&mut d1).await;
    assert_eq!(posts.len(), 0);

    let users = load_users(&mut d1).await;
    assert_eq!(users.len(), 4);
    assert_user(&users[0], 1, "John Doe", "2021-01-01");
    assert_user(&users[1], 2, "Jane Smith", "2021-01-02");
    assert_user(&users[2], 3, "Jim Beam", "2021-01-03");
    assert_user(&users[3], 4, "Jane Doe", "2021-01-04");
}

pub async fn test_delete_identifiable(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME, SessionOptions::default()).unwrap();

    let post: Post = sample_schema::posts::table
        .find(5)
        .select(sample_schema::posts::all_columns)
        .first(&mut d1)
        .await
        .unwrap();

    // Test: Del2 - Identifiable
    diesel::delete(&post).execute(&mut d1).await.unwrap();

    let posts = load_posts(&mut d1).await;
    assert_eq!(posts.len(), 4);
    assert_post(&posts[0], 1, "Hello World", Some("This is a test post"), 1);
    assert_post(
        &posts[1],
        2,
        "Another Post",
        Some("This is another test post"),
        1,
    );
    assert_post(&posts[2], 3, "Post #3", Some("Lots of words"), 2);
    assert_post(&posts[3], 4, "Post #4", Some("Even more words"), 2);
}

pub async fn test_delete_where(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME, SessionOptions::default()).unwrap();

    // Test: DelW1 - `=` int
    diesel::delete(sample_schema::posts::table.filter(sample_schema::posts::id.eq(5)))
        .execute(&mut d1)
        .await
        .unwrap();

    let posts = load_posts(&mut d1).await;
    assert_eq!(posts.len(), 4);
    assert_post(&posts[0], 1, "Hello World", Some("This is a test post"), 1);
    assert_post(
        &posts[1],
        2,
        "Another Post",
        Some("This is another test post"),
        1,
    );
    assert_post(&posts[2], 3, "Post #3", Some("Lots of words"), 2);
    assert_post(&posts[3], 4, "Post #4", Some("Even more words"), 2);

    // Test: DelW4 - `=` string
    diesel::delete(sample_schema::users::table.filter(sample_schema::users::name.eq("Jane Doe")))
        .execute(&mut d1)
        .await
        .unwrap();

    let users = load_users(&mut d1).await;
    assert_eq!(users.len(), 3);
    assert_user(&users[0], 1, "John Doe", "2021-01-01");
    assert_user(&users[1], 2, "Jane Smith", "2021-01-02");
    assert_user(&users[2], 3, "Jim Beam", "2021-01-03");

    // Test: DelW2 - `and`
    diesel::delete(
        sample_schema::posts::table.filter(
            sample_schema::posts::user_id
                .eq(2)
                .and(sample_schema::posts::id.eq(3)),
        ),
    )
    .execute(&mut d1)
    .await
    .unwrap();

    let posts = load_posts(&mut d1).await;
    assert_eq!(posts.len(), 3);
    assert_post(&posts[0], 1, "Hello World", Some("This is a test post"), 1);
    assert_post(
        &posts[1],
        2,
        "Another Post",
        Some("This is another test post"),
        1,
    );
    assert_post(&posts[2], 4, "Post #4", Some("Even more words"), 2);

    // Test: DelW3 - `in` int[]
    diesel::delete(sample_schema::posts::table.filter(sample_schema::posts::id.eq_any(vec![1, 2])))
        .execute(&mut d1)
        .await
        .unwrap();

    let posts = load_posts(&mut d1).await;
    assert_eq!(posts.len(), 1);
    assert_post(&posts[0], 4, "Post #4", Some("Even more words"), 2);
}

pub async fn test_delete_zero_rows(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME, SessionOptions::default()).unwrap();

    // Test: DelE2 - Zero rows
    let count =
        diesel::delete(sample_schema::posts::table.filter(sample_schema::posts::id.eq(999)))
            .execute(&mut d1)
            .await
            .unwrap();
    assert_eq!(count, 0);

    let posts = load_posts(&mut d1).await;
    assert_eq!(posts.len(), 5);
    assert_post(&posts[0], 1, "Hello World", Some("This is a test post"), 1);
    assert_post(
        &posts[1],
        2,
        "Another Post",
        Some("This is another test post"),
        1,
    );
    assert_post(&posts[2], 3, "Post #3", Some("Lots of words"), 2);
    assert_post(&posts[3], 4, "Post #4", Some("Even more words"), 2);
    assert_post(&posts[4], 5, "Post #5", Some("Imagine a post here"), 2);
}

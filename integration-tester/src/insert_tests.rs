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

    diesel::table! {
        notes (id) {
            id -> Integer,
            label -> Text,
        }
    }
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

#[derive(Queryable, Debug)]
struct Note {
    id: i32,
    label: String,
}

#[derive(Insertable)]
#[diesel(table_name = sample_schema::users)]
struct NewUser {
    id: i32,
    name: String,
    created_at: String,
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

pub async fn test_insert_values(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME, SessionOptions::default()).unwrap();

    // Test: I1 - All columns
    // Test: I3 - Tuple of columns
    let insert = diesel::insert_into(sample_schema::users::table).values((
        sample_schema::users::id.eq(5),
        sample_schema::users::name.eq("Ada Lovelace"),
        sample_schema::users::created_at.eq("2021-01-05"),
    ));

    // Test: IE1 - Affected row count
    let count = insert.execute(&mut d1).await.unwrap();
    assert_eq!(count, 1);

    // Test: I2 - Specific columns
    diesel::insert_into(sample_schema::users::table)
        .values((
            sample_schema::users::name.eq("Auto Id"),
            sample_schema::users::created_at.eq("2021-01-06"),
        ))
        .execute(&mut d1)
        .await
        .unwrap();

    // Test: I4 - Insertable
    diesel::insert_into(sample_schema::users::table)
        .values(NewUser {
            id: 7,
            name: "Grace Hopper".to_string(),
            created_at: "2021-01-07".to_string(),
        })
        .execute(&mut d1)
        .await
        .unwrap();

    let users = load_users(&mut d1).await;
    assert_eq!(users.len(), 7);
    assert_user(&users[0], 1, "John Doe", "2021-01-01");
    assert_user(&users[1], 2, "Jane Smith", "2021-01-02");
    assert_user(&users[2], 3, "Jim Beam", "2021-01-03");
    assert_user(&users[3], 4, "Jane Doe", "2021-01-04");
    assert_user(&users[4], 5, "Ada Lovelace", "2021-01-05");
    assert_user(&users[5], 6, "Auto Id", "2021-01-06");
    assert_user(&users[6], 7, "Grace Hopper", "2021-01-07");

    // Test: IV1 - `NULL`
    diesel::insert_into(sample_schema::posts::table)
        .values((
            sample_schema::posts::id.eq(6),
            sample_schema::posts::title.eq("No body"),
            sample_schema::posts::body.eq(None::<String>),
            sample_schema::posts::user_id.eq(1),
        ))
        .execute(&mut d1)
        .await
        .unwrap();

    // Test: IV2 - `Some`
    diesel::insert_into(sample_schema::posts::table)
        .values((
            sample_schema::posts::id.eq(7),
            sample_schema::posts::title.eq("Has body"),
            sample_schema::posts::body.eq(Some("written".to_string())),
            sample_schema::posts::user_id.eq(1),
        ))
        .execute(&mut d1)
        .await
        .unwrap();

    let posts = load_posts(&mut d1).await;
    assert_eq!(posts.len(), 7);
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
    assert_post(&posts[5], 6, "No body", None, 1);
    assert_post(&posts[6], 7, "Has body", Some("written"), 1);
}

pub async fn test_insert_default_values(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME, SessionOptions::default()).unwrap();

    // Test: I5 - Default values
    diesel::insert_into(sample_schema::notes::table)
        .default_values()
        .execute(&mut d1)
        .await
        .unwrap();

    let notes: Vec<Note> = sample_schema::notes::table
        .select(sample_schema::notes::all_columns)
        .order(sample_schema::notes::id)
        .load(&mut d1)
        .await
        .unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].id, 1);
    assert_eq!(notes[0].label, "untitled");
}

pub async fn test_insert_batch(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME, SessionOptions::default()).unwrap();

    let new_users = vec![
        NewUser {
            id: 5,
            name: "Batch One".to_string(),
            created_at: "2021-01-05".to_string(),
        },
        NewUser {
            id: 6,
            name: "Batch Two".to_string(),
            created_at: "2021-01-06".to_string(),
        },
    ];

    // Test: IB1 - Multiple rows
    let count = diesel::insert_into(sample_schema::users::table)
        .values(&new_users)
        .execute(&mut d1)
        .await
        .unwrap();
    assert_eq!(count, 2);

    let users = load_users(&mut d1).await;
    assert_eq!(users.len(), 6);
    assert_user(&users[0], 1, "John Doe", "2021-01-01");
    assert_user(&users[1], 2, "Jane Smith", "2021-01-02");
    assert_user(&users[2], 3, "Jim Beam", "2021-01-03");
    assert_user(&users[3], 4, "Jane Doe", "2021-01-04");
    assert_user(&users[4], 5, "Batch One", "2021-01-05");
    assert_user(&users[5], 6, "Batch Two", "2021-01-06");
}

pub async fn test_insert_from_select(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME, SessionOptions::default()).unwrap();

    // Test: IS1 - Insert from select
    diesel::insert_into(sample_schema::posts::table)
        .values(
            sample_schema::posts::table
                .filter(sample_schema::posts::id.eq(1))
                .select((
                    sample_schema::posts::title,
                    sample_schema::posts::body,
                    sample_schema::posts::user_id,
                )),
        )
        .into_columns((
            sample_schema::posts::title,
            sample_schema::posts::body,
            sample_schema::posts::user_id,
        ))
        .execute(&mut d1)
        .await
        .unwrap();

    let posts = load_posts(&mut d1).await;
    assert_eq!(posts.len(), 6);
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
    assert_post(&posts[5], 6, "Hello World", Some("This is a test post"), 1);
}

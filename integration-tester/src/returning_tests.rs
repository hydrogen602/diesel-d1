use diesel::prelude::*;
use diesel::upsert::excluded;
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

#[derive(Queryable, Debug)]
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

pub async fn test_returning_insert(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME, SessionOptions::default()).unwrap();

    // Test: R1 - INSERT all columns
    let user: User = diesel::insert_into(sample_schema::users::table)
        .values((
            sample_schema::users::id.eq(5),
            sample_schema::users::name.eq("Ada Lovelace"),
            sample_schema::users::created_at.eq("2021-01-05"),
        ))
        .returning(sample_schema::users::all_columns)
        .get_result(&mut d1)
        .await
        .unwrap();
    assert_user(&user, 5, "Ada Lovelace", "2021-01-05");

    // Test: R2 - INSERT specific columns
    let (id, title): (i32, String) = diesel::insert_into(sample_schema::posts::table)
        .values((
            sample_schema::posts::id.eq(6),
            sample_schema::posts::title.eq("Returned"),
            sample_schema::posts::body.eq(None::<String>),
            sample_schema::posts::user_id.eq(1),
        ))
        .returning((sample_schema::posts::id, sample_schema::posts::title))
        .get_result(&mut d1)
        .await
        .unwrap();
    assert_eq!(id, 6);
    assert_eq!(title, "Returned");

    let users = load_users(&mut d1).await;
    assert_eq!(users.len(), 5);
    assert_user(&users[0], 1, "John Doe", "2021-01-01");
    assert_user(&users[1], 2, "Jane Smith", "2021-01-02");
    assert_user(&users[2], 3, "Jim Beam", "2021-01-03");
    assert_user(&users[3], 4, "Jane Doe", "2021-01-04");
    assert_user(&users[4], 5, "Ada Lovelace", "2021-01-05");

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
    assert_post(&posts[5], 6, "Returned", None, 1);
}

pub async fn test_returning_update(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME, SessionOptions::default()).unwrap();

    // Test: R3 - UPDATE all columns
    let user: User =
        diesel::update(sample_schema::users::table.filter(sample_schema::users::id.eq(1)))
            .set(sample_schema::users::name.eq("Jack Doe"))
            .returning(sample_schema::users::all_columns)
            .get_result(&mut d1)
            .await
            .unwrap();
    assert_user(&user, 1, "Jack Doe", "2021-01-01");

    // Test: R4 - UPDATE specific columns
    let title: String =
        diesel::update(sample_schema::posts::table.filter(sample_schema::posts::id.eq(1)))
            .set(sample_schema::posts::title.eq("Hello"))
            .returning(sample_schema::posts::title)
            .get_result(&mut d1)
            .await
            .unwrap();
    assert_eq!(title, "Hello");

    // Test: R8 - Multiple rows
    let mut users: Vec<User> = diesel::update(
        sample_schema::users::table.filter(sample_schema::users::id.eq_any(vec![3, 4])),
    )
    .set(sample_schema::users::created_at.eq("2021-02-01"))
    .returning(sample_schema::users::all_columns)
    .get_results(&mut d1)
    .await
    .unwrap();
    users.sort_by_key(|row| row.id);
    assert_eq!(users.len(), 2);
    assert_user(&users[0], 3, "Jim Beam", "2021-02-01");
    assert_user(&users[1], 4, "Jane Doe", "2021-02-01");

    let users = load_users(&mut d1).await;
    assert_eq!(users.len(), 4);
    assert_user(&users[0], 1, "Jack Doe", "2021-01-01");
    assert_user(&users[1], 2, "Jane Smith", "2021-01-02");
    assert_user(&users[2], 3, "Jim Beam", "2021-02-01");
    assert_user(&users[3], 4, "Jane Doe", "2021-02-01");

    let posts = load_posts(&mut d1).await;
    assert_eq!(posts.len(), 5);
    assert_post(&posts[0], 1, "Hello", Some("This is a test post"), 1);
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

pub async fn test_returning_delete(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME, SessionOptions::default()).unwrap();

    // Test: R6 - DELETE specific columns
    let title: String =
        diesel::delete(sample_schema::posts::table.filter(sample_schema::posts::id.eq(5)))
            .returning(sample_schema::posts::title)
            .get_result(&mut d1)
            .await
            .unwrap();
    assert_eq!(title, "Post #5");

    // Test: R5 - DELETE all columns
    let user: User =
        diesel::delete(sample_schema::users::table.filter(sample_schema::users::id.eq(4)))
            .returning(sample_schema::users::all_columns)
            .get_result(&mut d1)
            .await
            .unwrap();
    assert_user(&user, 4, "Jane Doe", "2021-01-04");

    let users = load_users(&mut d1).await;
    assert_eq!(users.len(), 3);
    assert_user(&users[0], 1, "John Doe", "2021-01-01");
    assert_user(&users[1], 2, "Jane Smith", "2021-01-02");
    assert_user(&users[2], 3, "Jim Beam", "2021-01-03");

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

pub async fn test_returning_upsert(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME, SessionOptions::default()).unwrap();

    // Test: R7 - Upsert
    let inserted: User = diesel::insert_into(sample_schema::users::table)
        .values((
            sample_schema::users::id.eq(5),
            sample_schema::users::name.eq("Ada Lovelace"),
            sample_schema::users::created_at.eq("2021-01-05"),
        ))
        .on_conflict(sample_schema::users::id)
        .do_update()
        .set(sample_schema::users::name.eq(excluded(sample_schema::users::name)))
        .returning(sample_schema::users::all_columns)
        .get_result(&mut d1)
        .await
        .unwrap();
    assert_user(&inserted, 5, "Ada Lovelace", "2021-01-05");

    let updated: User = diesel::insert_into(sample_schema::users::table)
        .values((
            sample_schema::users::id.eq(5),
            sample_schema::users::name.eq("Ada"),
            sample_schema::users::created_at.eq("2099-01-01"),
        ))
        .on_conflict(sample_schema::users::id)
        .do_update()
        .set(sample_schema::users::name.eq(excluded(sample_schema::users::name)))
        .returning(sample_schema::users::all_columns)
        .get_result(&mut d1)
        .await
        .unwrap();
    assert_user(&updated, 5, "Ada", "2021-01-05");

    let users = load_users(&mut d1).await;
    assert_eq!(users.len(), 5);
    assert_user(&users[0], 1, "John Doe", "2021-01-01");
    assert_user(&users[1], 2, "Jane Smith", "2021-01-02");
    assert_user(&users[2], 3, "Jim Beam", "2021-01-03");
    assert_user(&users[3], 4, "Jane Doe", "2021-01-04");
    assert_user(&users[4], 5, "Ada", "2021-01-05");
}

pub async fn test_returning_zero_rows(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME, SessionOptions::default()).unwrap();

    // Test: RE1 - Zero rows
    let result =
        diesel::update(sample_schema::users::table.filter(sample_schema::users::id.eq(999)))
            .set(sample_schema::users::name.eq("Nobody"))
            .returning(sample_schema::users::all_columns)
            .get_result::<User>(&mut d1)
            .await;
    assert!(matches!(result, Err(diesel::result::Error::NotFound)));

    let users = load_users(&mut d1).await;
    assert_eq!(users.len(), 4);
    assert_user(&users[0], 1, "John Doe", "2021-01-01");
    assert_user(&users[1], 2, "Jane Smith", "2021-01-02");
    assert_user(&users[2], 3, "Jim Beam", "2021-01-03");
    assert_user(&users[3], 4, "Jane Doe", "2021-01-04");
}

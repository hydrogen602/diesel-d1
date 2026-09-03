use diesel::prelude::*;
use diesel_async::RunQueryDsl;
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

#[derive(Queryable, Identifiable, Debug)]
#[diesel(table_name = sample_schema::users)]
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

#[derive(AsChangeset)]
#[diesel(table_name = sample_schema::users)]
struct UserNameChange {
    name: String,
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

pub async fn test_update_all_rows(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME).unwrap();

    // Test: U1 - All rows
    let update = diesel::update(sample_schema::users::table)
        .set(sample_schema::users::created_at.eq("2021-12-31"));

    // Test: UE1 - Affected row count
    let count = update.execute(&mut d1).await.unwrap();
    assert_eq!(count, 4);

    let users = load_users(&mut d1).await;
    assert_eq!(users.len(), 4);
    assert_user(&users[0], 1, "John Doe", "2021-12-31");
    assert_user(&users[1], 2, "Jane Smith", "2021-12-31");
    assert_user(&users[2], 3, "Jim Beam", "2021-12-31");
    assert_user(&users[3], 4, "Jane Doe", "2021-12-31");
}

pub async fn test_update_columns(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME).unwrap();

    // Test: U2 - Single column
    diesel::update(sample_schema::posts::table.filter(sample_schema::posts::id.eq(1)))
        .set(sample_schema::posts::title.eq("Hello"))
        .execute(&mut d1)
        .await
        .unwrap();

    // Test: U3 - Multiple columns
    diesel::update(sample_schema::posts::table.filter(sample_schema::posts::id.eq(2)))
        .set((
            sample_schema::posts::title.eq("Renamed"),
            sample_schema::posts::body.eq(Some("rewritten".to_string())),
        ))
        .execute(&mut d1)
        .await
        .unwrap();

    let posts = load_posts(&mut d1).await;
    assert_eq!(posts.len(), 5);
    assert_post(&posts[0], 1, "Hello", Some("This is a test post"), 1);
    assert_post(&posts[1], 2, "Renamed", Some("rewritten"), 1);
    assert_post(&posts[2], 3, "Post #3", Some("Lots of words"), 2);
    assert_post(&posts[3], 4, "Post #4", Some("Even more words"), 2);
    assert_post(&posts[4], 5, "Post #5", Some("Imagine a post here"), 2);
}

pub async fn test_update_changeset(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME).unwrap();

    // Test: U4 - AsChangeset
    diesel::update(sample_schema::users::table.filter(sample_schema::users::id.eq(1)))
        .set(&UserNameChange {
            name: "Jack Doe".to_string(),
        })
        .execute(&mut d1)
        .await
        .unwrap();

    let user: User = sample_schema::users::table
        .find(2)
        .select(sample_schema::users::all_columns)
        .first(&mut d1)
        .await
        .unwrap();

    // Test: U5 - Identifiable
    diesel::update(&user)
        .set(sample_schema::users::name.eq("Janet Smith"))
        .execute(&mut d1)
        .await
        .unwrap();

    let users = load_users(&mut d1).await;
    assert_eq!(users.len(), 4);
    assert_user(&users[0], 1, "Jack Doe", "2021-01-01");
    assert_user(&users[1], 2, "Janet Smith", "2021-01-02");
    assert_user(&users[2], 3, "Jim Beam", "2021-01-03");
    assert_user(&users[3], 4, "Jane Doe", "2021-01-04");
}

pub async fn test_update_where(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME).unwrap();

    // Test: UW1 - `=` int
    diesel::update(sample_schema::users::table.filter(sample_schema::users::id.eq(1)))
        .set(sample_schema::users::name.eq("Jack Doe"))
        .execute(&mut d1)
        .await
        .unwrap();

    let users = load_users(&mut d1).await;
    assert_eq!(users.len(), 4);
    assert_user(&users[0], 1, "Jack Doe", "2021-01-01");
    assert_user(&users[1], 2, "Jane Smith", "2021-01-02");
    assert_user(&users[2], 3, "Jim Beam", "2021-01-03");
    assert_user(&users[3], 4, "Jane Doe", "2021-01-04");

    // Test: UW4 - `=` string
    diesel::update(sample_schema::users::table.filter(sample_schema::users::name.eq("Jim Beam")))
        .set(sample_schema::users::created_at.eq("2021-02-03"))
        .execute(&mut d1)
        .await
        .unwrap();

    let users = load_users(&mut d1).await;
    assert_eq!(users.len(), 4);
    assert_user(&users[0], 1, "Jack Doe", "2021-01-01");
    assert_user(&users[1], 2, "Jane Smith", "2021-01-02");
    assert_user(&users[2], 3, "Jim Beam", "2021-02-03");
    assert_user(&users[3], 4, "Jane Doe", "2021-01-04");

    // Test: UW3 - `in` int[]
    diesel::update(sample_schema::users::table.filter(sample_schema::users::id.eq_any(vec![2, 4])))
        .set(sample_schema::users::created_at.eq("2021-02-01"))
        .execute(&mut d1)
        .await
        .unwrap();

    let users = load_users(&mut d1).await;
    assert_eq!(users.len(), 4);
    assert_user(&users[0], 1, "Jack Doe", "2021-01-01");
    assert_user(&users[1], 2, "Jane Smith", "2021-02-01");
    assert_user(&users[2], 3, "Jim Beam", "2021-02-03");
    assert_user(&users[3], 4, "Jane Doe", "2021-02-01");

    // Test: UW2 - `and`
    diesel::update(
        sample_schema::users::table.filter(
            sample_schema::users::id
                .eq(2)
                .and(sample_schema::users::name.eq("Jane Smith")),
        ),
    )
    .set(sample_schema::users::name.eq("Janet Smith"))
    .execute(&mut d1)
    .await
    .unwrap();

    let users = load_users(&mut d1).await;
    assert_eq!(users.len(), 4);
    assert_user(&users[0], 1, "Jack Doe", "2021-01-01");
    assert_user(&users[1], 2, "Janet Smith", "2021-02-01");
    assert_user(&users[2], 3, "Jim Beam", "2021-02-03");
    assert_user(&users[3], 4, "Jane Doe", "2021-02-01");
}

pub async fn test_update_set(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME).unwrap();

    // Test: US1 - Set `NULL`
    diesel::update(sample_schema::posts::table.filter(sample_schema::posts::id.eq(1)))
        .set(sample_schema::posts::body.eq(None::<String>))
        .execute(&mut d1)
        .await
        .unwrap();

    // Test: US2 - Set `Some`
    diesel::update(sample_schema::posts::table.filter(sample_schema::posts::id.eq(2)))
        .set(sample_schema::posts::body.eq(Some("rewritten".to_string())))
        .execute(&mut d1)
        .await
        .unwrap();

    // Test: US3 - Expression
    diesel::update(sample_schema::posts::table.filter(sample_schema::posts::id.eq(3)))
        .set(sample_schema::posts::user_id.eq(sample_schema::posts::user_id + 1))
        .execute(&mut d1)
        .await
        .unwrap();

    let posts = load_posts(&mut d1).await;
    assert_eq!(posts.len(), 5);
    assert_post(&posts[0], 1, "Hello World", None, 1);
    assert_post(&posts[1], 2, "Another Post", Some("rewritten"), 1);
    assert_post(&posts[2], 3, "Post #3", Some("Lots of words"), 3);
    assert_post(&posts[3], 4, "Post #4", Some("Even more words"), 2);
    assert_post(&posts[4], 5, "Post #5", Some("Imagine a post here"), 2);
}

pub async fn test_update_zero_rows(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME).unwrap();

    // Test: UE2 - Zero rows
    let count =
        diesel::update(sample_schema::users::table.filter(sample_schema::users::id.eq(999)))
            .set(sample_schema::users::name.eq("Nobody"))
            .execute(&mut d1)
            .await
            .unwrap();
    assert_eq!(count, 0);

    let users = load_users(&mut d1).await;
    assert_eq!(users.len(), 4);
    assert_user(&users[0], 1, "John Doe", "2021-01-01");
    assert_user(&users[1], 2, "Jane Smith", "2021-01-02");
    assert_user(&users[2], 3, "Jim Beam", "2021-01-03");
    assert_user(&users[3], 4, "Jane Doe", "2021-01-04");
}

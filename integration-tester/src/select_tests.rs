//! See if there is a way to conditionally compile this

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

    diesel::joinable!(posts -> users (user_id));
    diesel::allow_tables_to_appear_in_same_query!(posts, users);

    diesel::table! {
        typed (id) {
            id -> Integer,
            score -> Double,
            payload -> Nullable<Binary>,
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

#[derive(QueryableByName, Debug)]
#[diesel(table_name = sample_schema::users)]
struct UserByName {
    id: i32,
    name: String,
    created_at: String,
}

#[derive(QueryableByName, Debug)]
struct UserIdByName {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    id: i32,
}

#[derive(QueryableByName, Debug)]
struct UserNameByName {
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
}

fn assert_user(row: &User, id: i32, name: &str, created_at: &str) {
    assert_eq!(row.id, id);
    assert_eq!(row.name, name);
    assert_eq!(row.created_at, created_at);
}

fn assert_user_by_name(row: &UserByName, id: i32, name: &str, created_at: &str) {
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

#[derive(Queryable, Debug)]
struct Typed {
    id: i32,
    score: f64,
    payload: Option<Vec<u8>>,
}

fn assert_typed(row: &Typed, id: i32, score: f64, payload: Option<&[u8]>) {
    assert_eq!(row.id, id);
    assert_eq!(row.score, score);
    assert_eq!(row.payload.as_deref(), payload);
}

pub async fn test_users(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME).unwrap();

    // Test: S1 - All columns
    // Test: SW3 - `<` int
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .filter(sample_schema::users::id.lt(3));

    // Test: SQ1 - Queryable
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
    let mut d1 = D1Connection::new(env, D1_NAME).unwrap();

    // Test: S1 - All columns
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

    // Test: S3 - Tuple of columns
    // Test: SJ1 - Inner join
    let query = sample_schema::posts::table
        .inner_join(sample_schema::users::table)
        .select((
            sample_schema::posts::all_columns,
            sample_schema::users::name,
        ));

    // Test: SQ3 - Tuple of Queryable
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

pub async fn test_users_no_posts(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME).unwrap();

    // Test: S1 - All columns
    // Test: SJ2 - Left join
    // Test: SW1 - `is null`
    let query = sample_schema::users::table
        .left_join(sample_schema::posts::table)
        .select(sample_schema::users::all_columns)
        .filter(sample_schema::posts::id.is_null());

    // Test: SQ1 - Queryable
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].id, 3);
    assert_eq!(rows[0].name, "Jim Beam");
    assert_eq!(rows[0].created_at, "2021-01-03");
    assert_eq!(rows[1].id, 4);
    assert_eq!(rows[1].name, "Jane Doe");
    assert_eq!(rows[1].created_at, "2021-01-04");
}

pub async fn test_select_specific_columns(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME).unwrap();

    // Test: S2 - Specific columns
    let query = sample_schema::users::table
        .select((sample_schema::users::id, sample_schema::users::name))
        .order(sample_schema::users::id);

    let rows: Vec<(i32, String)> = query.load(&mut d1).await.unwrap();

    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0], (1, "John Doe".to_string()));
    assert_eq!(rows[1], (2, "Jane Smith".to_string()));
    assert_eq!(rows[2], (3, "Jim Beam".to_string()));
    assert_eq!(rows[3], (4, "Jane Doe".to_string()));
}

pub async fn test_where_not_null(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME).unwrap();

    // Test: SW2 - `is not null`
    let query = sample_schema::posts::table
        .select(sample_schema::posts::all_columns)
        .filter(sample_schema::posts::body.is_not_null())
        .order(sample_schema::posts::id);

    let rows: Vec<Post> = query.load(&mut d1).await.unwrap();

    assert_eq!(rows.len(), 5);
    assert_post(&rows[0], 1, "Hello World", Some("This is a test post"), 1);
    assert_post(
        &rows[1],
        2,
        "Another Post",
        Some("This is another test post"),
        1,
    );
    assert_post(&rows[2], 3, "Post #3", Some("Lots of words"), 2);
    assert_post(&rows[3], 4, "Post #4", Some("Even more words"), 2);
    assert_post(&rows[4], 5, "Post #5", Some("Imagine a post here"), 2);
}

pub async fn test_where_int(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME).unwrap();

    // Test: SW4 - `>` int
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .filter(sample_schema::users::id.gt(2))
        .order(sample_schema::users::id);
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_user(&rows[0], 3, "Jim Beam", "2021-01-03");
    assert_user(&rows[1], 4, "Jane Doe", "2021-01-04");

    // Test: SW5 - `<=` int
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .filter(sample_schema::users::id.le(2))
        .order(sample_schema::users::id);
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_user(&rows[0], 1, "John Doe", "2021-01-01");
    assert_user(&rows[1], 2, "Jane Smith", "2021-01-02");

    // Test: SW6 - `>=` int
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .filter(sample_schema::users::id.ge(3))
        .order(sample_schema::users::id);
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_user(&rows[0], 3, "Jim Beam", "2021-01-03");
    assert_user(&rows[1], 4, "Jane Doe", "2021-01-04");

    // Test: SW7 - `=` int
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .filter(sample_schema::users::id.eq(2));
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_user(&rows[0], 2, "Jane Smith", "2021-01-02");

    // Test: SW8 - `<>` int
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .filter(sample_schema::users::id.ne(2))
        .order(sample_schema::users::id);
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 3);
    assert_user(&rows[0], 1, "John Doe", "2021-01-01");
    assert_user(&rows[1], 3, "Jim Beam", "2021-01-03");
    assert_user(&rows[2], 4, "Jane Doe", "2021-01-04");

    // Test: SW9 - `in` int[]
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .filter(sample_schema::users::id.eq_any(vec![1, 4]))
        .order(sample_schema::users::id);
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_user(&rows[0], 1, "John Doe", "2021-01-01");
    assert_user(&rows[1], 4, "Jane Doe", "2021-01-04");

    // Test: SW10 - `not in` int[]
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .filter(sample_schema::users::id.ne_all(vec![1, 4]))
        .order(sample_schema::users::id);
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_user(&rows[0], 2, "Jane Smith", "2021-01-02");
    assert_user(&rows[1], 3, "Jim Beam", "2021-01-03");

    // Test: SW11 - `between` int
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .filter(sample_schema::users::id.between(2, 3))
        .order(sample_schema::users::id);
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_user(&rows[0], 2, "Jane Smith", "2021-01-02");
    assert_user(&rows[1], 3, "Jim Beam", "2021-01-03");

    // Test: SW12 - `not between` int
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .filter(sample_schema::users::id.not_between(2, 3))
        .order(sample_schema::users::id);
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_user(&rows[0], 1, "John Doe", "2021-01-01");
    assert_user(&rows[1], 4, "Jane Doe", "2021-01-04");
}

pub async fn test_where_string(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME).unwrap();

    // Test: SW13 - `<` string
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .filter(sample_schema::users::name.lt("Jim Beam"))
        .order(sample_schema::users::id);
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_user(&rows[0], 2, "Jane Smith", "2021-01-02");
    assert_user(&rows[1], 4, "Jane Doe", "2021-01-04");

    // Test: SW14 - `>` string
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .filter(sample_schema::users::name.gt("Jim Beam"))
        .order(sample_schema::users::id);
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_user(&rows[0], 1, "John Doe", "2021-01-01");

    // Test: SW15 - `<=` string
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .filter(sample_schema::users::name.le("Jim Beam"))
        .order(sample_schema::users::id);
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 3);
    assert_user(&rows[0], 2, "Jane Smith", "2021-01-02");
    assert_user(&rows[1], 3, "Jim Beam", "2021-01-03");
    assert_user(&rows[2], 4, "Jane Doe", "2021-01-04");

    // Test: SW16 - `>=` string
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .filter(sample_schema::users::name.ge("Jim Beam"))
        .order(sample_schema::users::id);
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_user(&rows[0], 1, "John Doe", "2021-01-01");
    assert_user(&rows[1], 3, "Jim Beam", "2021-01-03");

    // Test: SW17 - `=` string
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .filter(sample_schema::users::name.eq("Jane Doe"));
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_user(&rows[0], 4, "Jane Doe", "2021-01-04");

    // Test: SW18 - `<>` string
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .filter(sample_schema::users::name.ne("Jane Doe"))
        .order(sample_schema::users::id);
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 3);
    assert_user(&rows[0], 1, "John Doe", "2021-01-01");
    assert_user(&rows[1], 2, "Jane Smith", "2021-01-02");
    assert_user(&rows[2], 3, "Jim Beam", "2021-01-03");

    // Test: SW19 - `in` string[]
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .filter(sample_schema::users::name.eq_any(vec!["John Doe", "Jim Beam"]))
        .order(sample_schema::users::id);
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_user(&rows[0], 1, "John Doe", "2021-01-01");
    assert_user(&rows[1], 3, "Jim Beam", "2021-01-03");

    // Test: SW20 - `not in` string[]
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .filter(sample_schema::users::name.ne_all(vec!["John Doe", "Jim Beam"]))
        .order(sample_schema::users::id);
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_user(&rows[0], 2, "Jane Smith", "2021-01-02");
    assert_user(&rows[1], 4, "Jane Doe", "2021-01-04");

    // Test: SW21 - `like` string
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .filter(sample_schema::users::name.like("Jane%"))
        .order(sample_schema::users::id);
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_user(&rows[0], 2, "Jane Smith", "2021-01-02");
    assert_user(&rows[1], 4, "Jane Doe", "2021-01-04");

    // Test: SW22 - `not like` string
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .filter(sample_schema::users::name.not_like("Jane%"))
        .order(sample_schema::users::id);
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_user(&rows[0], 1, "John Doe", "2021-01-01");
    assert_user(&rows[1], 3, "Jim Beam", "2021-01-03");
}

pub async fn test_where_compound(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME).unwrap();

    // Test: SW23 - `and`
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .filter(
            sample_schema::users::id
                .gt(1)
                .and(sample_schema::users::id.lt(4)),
        )
        .order(sample_schema::users::id);
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_user(&rows[0], 2, "Jane Smith", "2021-01-02");
    assert_user(&rows[1], 3, "Jim Beam", "2021-01-03");

    // Test: SW24 - `or`
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .filter(
            sample_schema::users::name
                .eq("John Doe")
                .or(sample_schema::users::name.eq("Jim Beam")),
        )
        .order(sample_schema::users::id);
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_user(&rows[0], 1, "John Doe", "2021-01-01");
    assert_user(&rows[1], 3, "Jim Beam", "2021-01-03");

    // Test: SW25 - `not`
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .filter(diesel::dsl::not(sample_schema::users::name.like("Jane%")))
        .order(sample_schema::users::id);
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_user(&rows[0], 1, "John Doe", "2021-01-01");
    assert_user(&rows[1], 3, "Jim Beam", "2021-01-03");
}

pub async fn test_where_other_types(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME).unwrap();

    // Test: SW26 - `<` real
    let query = sample_schema::typed::table
        .select(sample_schema::typed::all_columns)
        .filter(sample_schema::typed::score.lt(2.5))
        .order(sample_schema::typed::id);
    let rows: Vec<Typed> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_typed(&rows[0], 1, 1.5, Some(&[0x01]));

    // Test: SW27 - `>` real
    let query = sample_schema::typed::table
        .select(sample_schema::typed::all_columns)
        .filter(sample_schema::typed::score.gt(3.5))
        .order(sample_schema::typed::id);
    let rows: Vec<Typed> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_typed(&rows[0], 4, 4.5, Some(&[0x01, 0x02]));

    // Test: SW28 - `<=` real
    let query = sample_schema::typed::table
        .select(sample_schema::typed::all_columns)
        .filter(sample_schema::typed::score.le(2.5))
        .order(sample_schema::typed::id);
    let rows: Vec<Typed> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_typed(&rows[0], 1, 1.5, Some(&[0x01]));
    assert_typed(&rows[1], 2, 2.5, Some(&[0x02]));

    // Test: SW29 - `>=` real
    let query = sample_schema::typed::table
        .select(sample_schema::typed::all_columns)
        .filter(sample_schema::typed::score.ge(3.5))
        .order(sample_schema::typed::id);
    let rows: Vec<Typed> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_typed(&rows[0], 3, 3.5, None);
    assert_typed(&rows[1], 4, 4.5, Some(&[0x01, 0x02]));

    // Test: SW30 - `=` real
    let query = sample_schema::typed::table
        .select(sample_schema::typed::all_columns)
        .filter(sample_schema::typed::score.eq(2.5));
    let rows: Vec<Typed> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_typed(&rows[0], 2, 2.5, Some(&[0x02]));

    // Test: SW31 - `<>` real
    let query = sample_schema::typed::table
        .select(sample_schema::typed::all_columns)
        .filter(sample_schema::typed::score.ne(2.5))
        .order(sample_schema::typed::id);
    let rows: Vec<Typed> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 3);
    assert_typed(&rows[0], 1, 1.5, Some(&[0x01]));
    assert_typed(&rows[1], 3, 3.5, None);
    assert_typed(&rows[2], 4, 4.5, Some(&[0x01, 0x02]));

    // Test: SW32 - `in` real[]
    let query = sample_schema::typed::table
        .select(sample_schema::typed::all_columns)
        .filter(sample_schema::typed::score.eq_any(vec![1.5, 4.5]))
        .order(sample_schema::typed::id);
    let rows: Vec<Typed> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_typed(&rows[0], 1, 1.5, Some(&[0x01]));
    assert_typed(&rows[1], 4, 4.5, Some(&[0x01, 0x02]));

    // Test: SW33 - `not in` real[]
    let query = sample_schema::typed::table
        .select(sample_schema::typed::all_columns)
        .filter(sample_schema::typed::score.ne_all(vec![1.5, 4.5]))
        .order(sample_schema::typed::id);
    let rows: Vec<Typed> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_typed(&rows[0], 2, 2.5, Some(&[0x02]));
    assert_typed(&rows[1], 3, 3.5, None);

    // Test: SW34 - `between` real
    let query = sample_schema::typed::table
        .select(sample_schema::typed::all_columns)
        .filter(sample_schema::typed::score.between(2.0, 4.0))
        .order(sample_schema::typed::id);
    let rows: Vec<Typed> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_typed(&rows[0], 2, 2.5, Some(&[0x02]));
    assert_typed(&rows[1], 3, 3.5, None);

    // Test: SW35 - `not between` real
    let query = sample_schema::typed::table
        .select(sample_schema::typed::all_columns)
        .filter(sample_schema::typed::score.not_between(2.0, 4.0))
        .order(sample_schema::typed::id);
    let rows: Vec<Typed> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_typed(&rows[0], 1, 1.5, Some(&[0x01]));
    assert_typed(&rows[1], 4, 4.5, Some(&[0x01, 0x02]));

    // Test: SW36 - `=` blob
    let query = sample_schema::typed::table
        .select(sample_schema::typed::all_columns)
        .filter(sample_schema::typed::payload.eq(Some(vec![0x01])));
    let rows: Vec<Typed> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_typed(&rows[0], 1, 1.5, Some(&[0x01]));

    // Test: SW37 - `<>` blob
    let query = sample_schema::typed::table
        .select(sample_schema::typed::all_columns)
        .filter(sample_schema::typed::payload.ne(Some(vec![0x01])))
        .order(sample_schema::typed::id);
    let rows: Vec<Typed> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_typed(&rows[0], 2, 2.5, Some(&[0x02]));
    assert_typed(&rows[1], 4, 4.5, Some(&[0x01, 0x02]));

    // Test: SW38 - `in` blob[]
    let query = sample_schema::typed::table
        .select(sample_schema::typed::all_columns)
        .filter(
            sample_schema::typed::payload.eq_any(vec![Some(vec![0x01]), Some(vec![0x01, 0x02])]),
        )
        .order(sample_schema::typed::id);
    let rows: Vec<Typed> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_typed(&rows[0], 1, 1.5, Some(&[0x01]));
    assert_typed(&rows[1], 4, 4.5, Some(&[0x01, 0x02]));

    // Test: SW39 - `is null`
    let query = sample_schema::typed::table
        .select(sample_schema::typed::all_columns)
        .filter(sample_schema::typed::payload.is_null());
    let rows: Vec<Typed> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_typed(&rows[0], 3, 3.5, None);

    // Test: SW40 - `is not null`
    let query = sample_schema::typed::table
        .select(sample_schema::typed::all_columns)
        .filter(sample_schema::typed::payload.is_not_null())
        .order(sample_schema::typed::id);
    let rows: Vec<Typed> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 3);
    assert_typed(&rows[0], 1, 1.5, Some(&[0x01]));
    assert_typed(&rows[1], 2, 2.5, Some(&[0x02]));
    assert_typed(&rows[2], 4, 4.5, Some(&[0x01, 0x02]));
}

pub async fn test_order_by(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME).unwrap();

    // Test: SO1 - Single column
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .order(sample_schema::users::name);
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 4);
    assert_user(&rows[0], 4, "Jane Doe", "2021-01-04");
    assert_user(&rows[1], 2, "Jane Smith", "2021-01-02");
    assert_user(&rows[2], 3, "Jim Beam", "2021-01-03");
    assert_user(&rows[3], 1, "John Doe", "2021-01-01");

    // Test: SO2 - Multiple columns
    let query = sample_schema::posts::table
        .select(sample_schema::posts::all_columns)
        .order((sample_schema::posts::user_id, sample_schema::posts::title));
    let rows: Vec<Post> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 5);
    assert_post(
        &rows[0],
        2,
        "Another Post",
        Some("This is another test post"),
        1,
    );
    assert_post(&rows[1], 1, "Hello World", Some("This is a test post"), 1);
    assert_post(&rows[2], 3, "Post #3", Some("Lots of words"), 2);
    assert_post(&rows[3], 4, "Post #4", Some("Even more words"), 2);
    assert_post(&rows[4], 5, "Post #5", Some("Imagine a post here"), 2);

    // Test: SO3 - Direction
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .order(sample_schema::users::id.desc());
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 4);
    assert_user(&rows[0], 4, "Jane Doe", "2021-01-04");
    assert_user(&rows[1], 3, "Jim Beam", "2021-01-03");
    assert_user(&rows[2], 2, "Jane Smith", "2021-01-02");
    assert_user(&rows[3], 1, "John Doe", "2021-01-01");
}

pub async fn test_limit_offset(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME).unwrap();

    // Test: SL1 - Single value
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .order(sample_schema::users::id)
        .limit(2);
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_user(&rows[0], 1, "John Doe", "2021-01-01");
    assert_user(&rows[1], 2, "Jane Smith", "2021-01-02");

    // Test: SOff1 - Single value
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .order(sample_schema::users::id)
        .offset(2);
    let rows: Vec<User> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_user(&rows[0], 3, "Jim Beam", "2021-01-03");
    assert_user(&rows[1], 4, "Jane Doe", "2021-01-04");
}

pub async fn test_joins_on(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME).unwrap();

    // Test: SJ3 - Left outer join
    let query = sample_schema::users::table
        .left_outer_join(sample_schema::posts::table)
        .select((
            sample_schema::users::all_columns,
            sample_schema::posts::all_columns.nullable(),
        ))
        .order((sample_schema::users::id, sample_schema::posts::id));

    // Test: SQ3 - Tuple of Queryable
    let rows: Vec<(User, Option<Post>)> = query.load(&mut d1).await.unwrap();

    assert_eq!(rows.len(), 7);
    assert_user(&rows[0].0, 1, "John Doe", "2021-01-01");
    assert_post(
        rows[0].1.as_ref().unwrap(),
        1,
        "Hello World",
        Some("This is a test post"),
        1,
    );
    assert_user(&rows[1].0, 1, "John Doe", "2021-01-01");
    assert_post(
        rows[1].1.as_ref().unwrap(),
        2,
        "Another Post",
        Some("This is another test post"),
        1,
    );
    assert_user(&rows[2].0, 2, "Jane Smith", "2021-01-02");
    assert_post(
        rows[2].1.as_ref().unwrap(),
        3,
        "Post #3",
        Some("Lots of words"),
        2,
    );
    assert_user(&rows[3].0, 2, "Jane Smith", "2021-01-02");
    assert_post(
        rows[3].1.as_ref().unwrap(),
        4,
        "Post #4",
        Some("Even more words"),
        2,
    );
    assert_user(&rows[4].0, 2, "Jane Smith", "2021-01-02");
    assert_post(
        rows[4].1.as_ref().unwrap(),
        5,
        "Post #5",
        Some("Imagine a post here"),
        2,
    );
    assert_user(&rows[5].0, 3, "Jim Beam", "2021-01-03");
    assert!(rows[5].1.is_none());
    assert_user(&rows[6].0, 4, "Jane Doe", "2021-01-04");
    assert!(rows[6].1.is_none());

    // Test: SJ4 - Inner join ON
    let query = sample_schema::posts::table
        .inner_join(
            sample_schema::users::table.on(sample_schema::posts::user_id
                .eq(sample_schema::users::id)
                .and(sample_schema::users::id.eq(1))),
        )
        .select((
            sample_schema::posts::all_columns,
            sample_schema::users::name,
        ))
        .order(sample_schema::posts::id);
    let rows: Vec<(Post, String)> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_post(&rows[0].0, 1, "Hello World", Some("This is a test post"), 1);
    assert_eq!(rows[0].1, "John Doe");
    assert_post(
        &rows[1].0,
        2,
        "Another Post",
        Some("This is another test post"),
        1,
    );
    assert_eq!(rows[1].1, "John Doe");

    // Test: SJ5 - Left join ON
    let query = sample_schema::users::table
        .left_join(
            sample_schema::posts::table.on(sample_schema::posts::user_id
                .eq(sample_schema::users::id)
                .and(sample_schema::posts::id.eq(1))),
        )
        .select((
            sample_schema::users::all_columns,
            sample_schema::posts::all_columns.nullable(),
        ))
        .order(sample_schema::users::id);
    let rows: Vec<(User, Option<Post>)> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 4);
    assert_user(&rows[0].0, 1, "John Doe", "2021-01-01");
    assert_post(
        rows[0].1.as_ref().unwrap(),
        1,
        "Hello World",
        Some("This is a test post"),
        1,
    );
    assert_user(&rows[1].0, 2, "Jane Smith", "2021-01-02");
    assert!(rows[1].1.is_none());
    assert_user(&rows[2].0, 3, "Jim Beam", "2021-01-03");
    assert!(rows[2].1.is_none());
    assert_user(&rows[3].0, 4, "Jane Doe", "2021-01-04");
    assert!(rows[3].1.is_none());

    // Test: SJ6 - Left outer join ON
    let query = sample_schema::users::table
        .left_outer_join(
            sample_schema::posts::table.on(sample_schema::posts::user_id
                .eq(sample_schema::users::id)
                .and(sample_schema::posts::title.like("Post%"))),
        )
        .select((
            sample_schema::users::all_columns,
            sample_schema::posts::all_columns.nullable(),
        ))
        .order((sample_schema::users::id, sample_schema::posts::id));
    let rows: Vec<(User, Option<Post>)> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 6);
    assert_user(&rows[0].0, 1, "John Doe", "2021-01-01");
    assert!(rows[0].1.is_none());
    assert_user(&rows[1].0, 2, "Jane Smith", "2021-01-02");
    assert_post(
        rows[1].1.as_ref().unwrap(),
        3,
        "Post #3",
        Some("Lots of words"),
        2,
    );
    assert_user(&rows[2].0, 2, "Jane Smith", "2021-01-02");
    assert_post(
        rows[2].1.as_ref().unwrap(),
        4,
        "Post #4",
        Some("Even more words"),
        2,
    );
    assert_user(&rows[3].0, 2, "Jane Smith", "2021-01-02");
    assert_post(
        rows[3].1.as_ref().unwrap(),
        5,
        "Post #5",
        Some("Imagine a post here"),
        2,
    );
    assert_user(&rows[4].0, 3, "Jim Beam", "2021-01-03");
    assert!(rows[4].1.is_none());
    assert_user(&rows[5].0, 4, "Jane Doe", "2021-01-04");
    assert!(rows[5].1.is_none());
}

pub async fn test_joins_on_non_fk(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME).unwrap();

    // Test: SJ7 - Inner join ON non-FK
    // Test: SJOn1 - `=` non-FK int
    let query = sample_schema::posts::table
        .inner_join(
            sample_schema::users::table.on(sample_schema::posts::id.eq(sample_schema::users::id)),
        )
        .select((
            sample_schema::posts::all_columns,
            sample_schema::users::all_columns,
        ))
        .order(sample_schema::posts::id);
    let rows: Vec<(Post, User)> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 4);
    assert_post(&rows[0].0, 1, "Hello World", Some("This is a test post"), 1);
    assert_user(&rows[0].1, 1, "John Doe", "2021-01-01");
    assert_post(
        &rows[1].0,
        2,
        "Another Post",
        Some("This is another test post"),
        1,
    );
    assert_user(&rows[1].1, 2, "Jane Smith", "2021-01-02");
    assert_post(&rows[2].0, 3, "Post #3", Some("Lots of words"), 2);
    assert_user(&rows[2].1, 3, "Jim Beam", "2021-01-03");
    assert_post(&rows[3].0, 4, "Post #4", Some("Even more words"), 2);
    assert_user(&rows[3].1, 4, "Jane Doe", "2021-01-04");

    // Test: SJ8 - Left join ON non-FK
    let query = sample_schema::posts::table
        .left_join(
            sample_schema::users::table.on(sample_schema::posts::id.eq(sample_schema::users::id)),
        )
        .select((
            sample_schema::posts::all_columns,
            sample_schema::users::all_columns.nullable(),
        ))
        .order(sample_schema::posts::id);
    let rows: Vec<(Post, Option<User>)> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 5);
    assert_post(&rows[0].0, 1, "Hello World", Some("This is a test post"), 1);
    assert_user(rows[0].1.as_ref().unwrap(), 1, "John Doe", "2021-01-01");
    assert_post(
        &rows[1].0,
        2,
        "Another Post",
        Some("This is another test post"),
        1,
    );
    assert_user(rows[1].1.as_ref().unwrap(), 2, "Jane Smith", "2021-01-02");
    assert_post(&rows[2].0, 3, "Post #3", Some("Lots of words"), 2);
    assert_user(rows[2].1.as_ref().unwrap(), 3, "Jim Beam", "2021-01-03");
    assert_post(&rows[3].0, 4, "Post #4", Some("Even more words"), 2);
    assert_user(rows[3].1.as_ref().unwrap(), 4, "Jane Doe", "2021-01-04");
    assert_post(&rows[4].0, 5, "Post #5", Some("Imagine a post here"), 2);
    assert!(rows[4].1.is_none());

    // Test: SJOn2 - Expression
    let query = sample_schema::posts::table
        .inner_join(
            sample_schema::users::table
                .on(sample_schema::posts::id.eq(sample_schema::users::id + 1)),
        )
        .select((
            sample_schema::posts::all_columns,
            sample_schema::users::all_columns,
        ))
        .order(sample_schema::posts::id);
    let rows: Vec<(Post, User)> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 4);
    assert_post(
        &rows[0].0,
        2,
        "Another Post",
        Some("This is another test post"),
        1,
    );
    assert_user(&rows[0].1, 1, "John Doe", "2021-01-01");
    assert_post(&rows[1].0, 3, "Post #3", Some("Lots of words"), 2);
    assert_user(&rows[1].1, 2, "Jane Smith", "2021-01-02");
    assert_post(&rows[2].0, 4, "Post #4", Some("Even more words"), 2);
    assert_user(&rows[2].1, 3, "Jim Beam", "2021-01-03");
    assert_post(&rows[3].0, 5, "Post #5", Some("Imagine a post here"), 2);
    assert_user(&rows[3].1, 4, "Jane Doe", "2021-01-04");

    // Test: SJOn3 - `=` string
    let query = sample_schema::posts::table
        .inner_join(sample_schema::users::table.on(sample_schema::users::name.eq("Jim Beam")))
        .select((
            sample_schema::posts::all_columns,
            sample_schema::users::name,
        ))
        .order(sample_schema::posts::id);
    let rows: Vec<(Post, String)> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 5);
    assert_post(&rows[0].0, 1, "Hello World", Some("This is a test post"), 1);
    assert_eq!(rows[0].1, "Jim Beam");
    assert_post(
        &rows[1].0,
        2,
        "Another Post",
        Some("This is another test post"),
        1,
    );
    assert_eq!(rows[1].1, "Jim Beam");
    assert_post(&rows[2].0, 3, "Post #3", Some("Lots of words"), 2);
    assert_eq!(rows[2].1, "Jim Beam");
    assert_post(&rows[3].0, 4, "Post #4", Some("Even more words"), 2);
    assert_eq!(rows[3].1, "Jim Beam");
    assert_post(&rows[4].0, 5, "Post #5", Some("Imagine a post here"), 2);
    assert_eq!(rows[4].1, "Jim Beam");

    // Test: SJOn4 - `and`
    let query = sample_schema::posts::table
        .inner_join(
            sample_schema::users::table.on(sample_schema::posts::id
                .eq(sample_schema::users::id)
                .and(sample_schema::users::name.like("Jane%"))),
        )
        .select((
            sample_schema::posts::all_columns,
            sample_schema::users::all_columns,
        ))
        .order(sample_schema::posts::id);
    let rows: Vec<(Post, User)> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_post(
        &rows[0].0,
        2,
        "Another Post",
        Some("This is another test post"),
        1,
    );
    assert_user(&rows[0].1, 2, "Jane Smith", "2021-01-02");
    assert_post(&rows[1].0, 4, "Post #4", Some("Even more words"), 2);
    assert_user(&rows[1].1, 4, "Jane Doe", "2021-01-04");

    let (left_users, right_users) = diesel::alias!(
        sample_schema::users as left_users,
        sample_schema::users as right_users
    );

    // Test: SJ9 - Self join ON
    let query = left_users
        .inner_join(
            right_users.on(left_users
                .field(sample_schema::users::id)
                .lt(right_users.field(sample_schema::users::id))
                .and(left_users.field(sample_schema::users::name).like("%Doe%"))
                .and(right_users.field(sample_schema::users::name).like("%Doe%"))),
        )
        .select((
            left_users.fields(sample_schema::users::all_columns),
            right_users.fields(sample_schema::users::all_columns),
        ));
    let rows: Vec<(User, User)> = query.load(&mut d1).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_user(&rows[0].0, 1, "John Doe", "2021-01-01");
    assert_user(&rows[0].1, 4, "Jane Doe", "2021-01-04");
}

pub async fn test_queryable_by_name(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME).unwrap();

    // Test: SQ2 - QueryableByName
    let rows: Vec<UserByName> = diesel::sql_query("SELECT * FROM users ORDER BY id")
        .load(&mut d1)
        .await
        .unwrap();
    assert_eq!(rows.len(), 4);
    assert_user_by_name(&rows[0], 1, "John Doe", "2021-01-01");
    assert_user_by_name(&rows[1], 2, "Jane Smith", "2021-01-02");
    assert_user_by_name(&rows[2], 3, "Jim Beam", "2021-01-03");
    assert_user_by_name(&rows[3], 4, "Jane Doe", "2021-01-04");

    // Test: SQ4 - Tuple of QueryableByName
    let rows: Vec<(UserIdByName, UserNameByName)> =
        diesel::sql_query("SELECT id, name FROM users ORDER BY id")
            .load(&mut d1)
            .await
            .unwrap();
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0].0.id, 1);
    assert_eq!(rows[0].1.name, "John Doe");
    assert_eq!(rows[1].0.id, 2);
    assert_eq!(rows[1].1.name, "Jane Smith");
    assert_eq!(rows[2].0.id, 3);
    assert_eq!(rows[2].1.name, "Jim Beam");
    assert_eq!(rows[3].0.id, 4);
    assert_eq!(rows[3].1.name, "Jane Doe");
}

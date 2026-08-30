//! See if there is a way to conditionally compile this

use diesel::{deserialize::Queryable, query_dsl::methods::SelectDsl};
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

#[event(scheduled)]
async fn main(event: ScheduledEvent, env: Env, ctx: ScheduleContext) {
    let mut d1 = D1Connection::new(env, "diesel_d1_test").unwrap();

    test_users(&mut d1).await;
}

async fn test_users(d1: &mut D1Connection) {
    let query = sample_schema::users::table.select(sample_schema::users::all_columns);

    let rows: Vec<User> = query.load(d1).await.unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].id, 1);
    assert_eq!(rows[0].name, "John Doe");
    assert_eq!(rows[0].created_at, "2021-01-01");
    assert_eq!(rows[1].id, 2);
    assert_eq!(rows[1].name, "Jane Smith");
    assert_eq!(rows[1].created_at, "2021-01-02");
}

use diesel::prelude::*;
use diesel::query_dsl::methods::FilterDsl;
use diesel::upsert::excluded;
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
}

#[derive(Queryable, Debug)]
struct User {
    id: i32,
    name: String,
    created_at: String,
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

async fn load_users(d1: &mut D1Connection) -> Vec<User> {
    sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .order(sample_schema::users::id)
        .load(d1)
        .await
        .unwrap()
}

pub async fn test_upsert_do_nothing(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME).unwrap();

    // Test: Up1 - DO NOTHING
    let ignored = diesel::insert_into(sample_schema::users::table)
        .values((
            sample_schema::users::id.eq(1),
            sample_schema::users::name.eq("Ignored"),
            sample_schema::users::created_at.eq("2099-01-01"),
        ))
        .on_conflict_do_nothing();

    // Test: UpE2 - Conflict no-op count
    let count = ignored.execute(&mut d1).await.unwrap();
    assert_eq!(count, 0);

    let users = load_users(&mut d1).await;
    assert_eq!(users.len(), 4);
    assert_user(&users[0], 1, "John Doe", "2021-01-01");
    assert_user(&users[1], 2, "Jane Smith", "2021-01-02");
    assert_user(&users[2], 3, "Jim Beam", "2021-01-03");
    assert_user(&users[3], 4, "Jane Doe", "2021-01-04");

    // Test: Up2 - DO NOTHING on target
    let inserted = diesel::insert_into(sample_schema::users::table)
        .values((
            sample_schema::users::id.eq(5),
            sample_schema::users::name.eq("Ada Lovelace"),
            sample_schema::users::created_at.eq("2021-01-05"),
        ))
        .on_conflict(sample_schema::users::id)
        .do_nothing();

    // Test: UpE1 - Inserted row count
    let count = inserted.execute(&mut d1).await.unwrap();
    assert_eq!(count, 1);

    diesel::insert_into(sample_schema::users::table)
        .values((
            sample_schema::users::id.eq(5),
            sample_schema::users::name.eq("Ignored"),
            sample_schema::users::created_at.eq("2099-01-01"),
        ))
        .on_conflict(sample_schema::users::id)
        .do_nothing()
        .execute(&mut d1)
        .await
        .unwrap();

    let users = load_users(&mut d1).await;
    assert_eq!(users.len(), 5);
    assert_user(&users[0], 1, "John Doe", "2021-01-01");
    assert_user(&users[1], 2, "Jane Smith", "2021-01-02");
    assert_user(&users[2], 3, "Jim Beam", "2021-01-03");
    assert_user(&users[3], 4, "Jane Doe", "2021-01-04");
    assert_user(&users[4], 5, "Ada Lovelace", "2021-01-05");
}

pub async fn test_upsert_do_update(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME).unwrap();

    // Test: Up3 - DO UPDATE SET
    diesel::insert_into(sample_schema::users::table)
        .values((
            sample_schema::users::id.eq(1),
            sample_schema::users::name.eq("Jack Doe"),
            sample_schema::users::created_at.eq("2099-01-01"),
        ))
        .on_conflict(sample_schema::users::id)
        .do_update()
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

    // Test: Up4 - excluded
    diesel::insert_into(sample_schema::users::table)
        .values((
            sample_schema::users::id.eq(2),
            sample_schema::users::name.eq("Janet Smith"),
            sample_schema::users::created_at.eq("2021-02-02"),
        ))
        .on_conflict(sample_schema::users::id)
        .do_update()
        .set((
            sample_schema::users::name.eq(excluded(sample_schema::users::name)),
            sample_schema::users::created_at.eq(excluded(sample_schema::users::created_at)),
        ))
        .execute(&mut d1)
        .await
        .unwrap();

    let users = load_users(&mut d1).await;
    assert_eq!(users.len(), 4);
    assert_user(&users[0], 1, "Jack Doe", "2021-01-01");
    assert_user(&users[1], 2, "Janet Smith", "2021-02-02");
    assert_user(&users[2], 3, "Jim Beam", "2021-01-03");
    assert_user(&users[3], 4, "Jane Doe", "2021-01-04");
}

pub async fn test_upsert_batch(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME).unwrap();

    let rows = vec![
        NewUser {
            id: 1,
            name: "Jack Doe".to_string(),
            created_at: "2021-01-01".to_string(),
        },
        NewUser {
            id: 5,
            name: "Ada Lovelace".to_string(),
            created_at: "2021-01-05".to_string(),
        },
    ];

    // Test: UpB1 - Multiple rows
    diesel::insert_into(sample_schema::users::table)
        .values(&rows)
        .on_conflict(sample_schema::users::id)
        .do_update()
        .set(sample_schema::users::name.eq(excluded(sample_schema::users::name)))
        .execute(&mut d1)
        .await
        .unwrap();

    let users = load_users(&mut d1).await;
    assert_eq!(users.len(), 5);
    assert_user(&users[0], 1, "Jack Doe", "2021-01-01");
    assert_user(&users[1], 2, "Jane Smith", "2021-01-02");
    assert_user(&users[2], 3, "Jim Beam", "2021-01-03");
    assert_user(&users[3], 4, "Jane Doe", "2021-01-04");
    assert_user(&users[4], 5, "Ada Lovelace", "2021-01-05");
}

pub async fn test_upsert_where(env: &Env) {
    let mut d1 = D1Connection::new(env, D1_NAME).unwrap();

    // Test: UpW1 - WHERE on DO UPDATE
    diesel::insert_into(sample_schema::users::table)
        .values((
            sample_schema::users::id.eq(1),
            sample_schema::users::name.eq("Should not apply"),
            sample_schema::users::created_at.eq("2099-01-01"),
        ))
        .on_conflict(sample_schema::users::id)
        .do_update()
        .set(sample_schema::users::name.eq("Should not apply"))
        .filter(sample_schema::users::name.eq("Jane Smith"))
        .execute(&mut d1)
        .await
        .unwrap();

    let users = load_users(&mut d1).await;
    assert_eq!(users.len(), 4);
    assert_user(&users[0], 1, "John Doe", "2021-01-01");
    assert_user(&users[1], 2, "Jane Smith", "2021-01-02");
    assert_user(&users[2], 3, "Jim Beam", "2021-01-03");
    assert_user(&users[3], 4, "Jane Doe", "2021-01-04");

    diesel::insert_into(sample_schema::users::table)
        .values((
            sample_schema::users::id.eq(1),
            sample_schema::users::name.eq("Jack Doe"),
            sample_schema::users::created_at.eq("2099-01-01"),
        ))
        .on_conflict(sample_schema::users::id)
        .do_update()
        .set(sample_schema::users::name.eq("Jack Doe"))
        .filter(sample_schema::users::name.eq("John Doe"))
        .execute(&mut d1)
        .await
        .unwrap();

    let users = load_users(&mut d1).await;
    assert_eq!(users.len(), 4);
    assert_user(&users[0], 1, "Jack Doe", "2021-01-01");
    assert_user(&users[1], 2, "Jane Smith", "2021-01-02");
    assert_user(&users[2], 3, "Jim Beam", "2021-01-03");
    assert_user(&users[3], 4, "Jane Doe", "2021-01-04");
}

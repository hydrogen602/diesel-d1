use diesel_d1::D1Connection;
use worker::*;

use crate::basic_tests::*;

mod basic_tests;

macro_rules! tests {
    ($($test_name:ident),*) => {
        async fn test_selector(test_to_run: &str, env: &Env) -> Result<Response> {
            match test_to_run {
                $(
                    stringify!($test_name) => $test_name(env).await,
                )*
                _ => return Response::error("Not found", 404),
            }
            Response::ok(format!("{} passed", test_to_run))
        }

        pub const ALL_TESTS: &'static [&'static str] = &[
            $(
                stringify!($test_name),
            )*
        ];
    };
}

tests!(
    test_users,
    test_posts,
    test_users_no_posts,
    test_select_specific_columns,
    test_where_not_null,
    test_where_int,
    test_where_string,
    test_where_compound,
    test_order_by,
    test_limit_offset,
    test_joins_on,
    test_queryable_by_name
);

pub const D1_NAME: &str = "diesel_d1_test";

async fn setup_d1(env: &Env) {
    let d1 = env.d1(D1_NAME).unwrap();
    let query = include_str!("../test_setup.sql");
    // D1 exec can't handle newlines within a single sql statement
    // so we remove all newlines and then add them back in only after each ;
    let query = query.replace('\n', "").replace(';', ";\n");
    d1.exec(&query).await.unwrap();
}

#[event(fetch)]
/// This will be loaded in a worker, and based on what http request is made,
/// it will call the appropriate test function.
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    let test_name = req.path();
    let test_name = test_name.as_str().strip_prefix('/').unwrap();

    setup_d1(&env).await;

    test_selector(test_name, &env).await
}

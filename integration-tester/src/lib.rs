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

tests!(test_users, test_posts);

#[event(fetch)]
/// This will be loaded in a worker, and based on what http request is made,
/// it will call the appropriate test function.
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    let test_name = req.path();
    let test_name = test_name.as_str().strip_prefix('/').unwrap();

    test_selector(test_name, &env).await
}

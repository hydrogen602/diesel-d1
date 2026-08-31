use diesel_d1::D1Connection;
use worker::*;

use crate::basic_tests::test_users;

mod basic_tests;

#[event(fetch)]
/// This will be loaded in a worker, and based on what http request is made,
/// it will call the appropriate test function.
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    let test_name = req.path();
    let test_name = test_name.as_str().strip_prefix('/').unwrap();
    match test_name {
        "test_users" => test_users(&env).await,
        _ => return Response::error("Not found", 404),
    }

    Response::ok("Hello, World!")
}

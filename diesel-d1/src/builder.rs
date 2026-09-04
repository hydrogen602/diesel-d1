use diesel_d1_core::{D1TransactionManager, Missing, Present, Required};
use worker::Env;

use crate::{D1Connection, SessionOptions};

mod private {}

#[derive(Default)]
/// Builder for a D1 connection.
pub struct D1ConnectionBuilder<E, N> {
    transaction_manager: D1TransactionManager,
    session: SessionOptions,
    env: E,
    name: N,
}

impl D1ConnectionBuilder<Missing, Missing> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'a, N: Required<&'a str>> D1ConnectionBuilder<Missing, N> {
    pub fn env(self, env: &Env) -> D1ConnectionBuilder<Present<&Env>, N> {
        let Self {
            transaction_manager,
            session,
            name,
            env: Missing,
        } = self;
        D1ConnectionBuilder {
            transaction_manager,
            session,
            name,
            env: Present(env),
        }
    }
}

impl<'a, E: Required<&'a Env>> D1ConnectionBuilder<E, Missing> {
    pub fn name(self, name: &str) -> D1ConnectionBuilder<E, Present<&str>> {
        let Self {
            transaction_manager,
            session,
            name: Missing,
            env,
        } = self;

        D1ConnectionBuilder {
            transaction_manager,
            session,
            name: Present(name),
            env,
        }
    }
}

impl<'a, E: Required<&'a Env>, N: Required<&'a str>> D1ConnectionBuilder<E, N> {
    pub fn transaction_manager(mut self, transaction_manager: D1TransactionManager) -> Self {
        self.transaction_manager = transaction_manager;
        self
    }

    pub fn session_option(mut self, session: SessionOptions) -> Self {
        self.session = session;
        self
    }
}

impl D1ConnectionBuilder<Present<&Env>, Present<&str>> {
    pub fn build(self) -> worker::Result<D1Connection> {
        let Self {
            transaction_manager,
            session,
            env: Present(env),
            name: Present(name),
        } = self;

        D1Connection::new(env, name, session, transaction_manager)
    }
}

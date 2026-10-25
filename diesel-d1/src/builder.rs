use worker::D1Database;

use crate::{
    D1Connection, SessionOptions,
    transaction_manager::D1TransactionManager,
    utils::{Missing, Present, Required},
};

mod private {}

#[derive(Default)]
/// Builder for a D1 connection.
pub struct D1ConnectionBuilder<D> {
    transaction_manager: D1TransactionManager,
    session: SessionOptions,
    binding: D,
}

impl D1ConnectionBuilder<Missing> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'a> D1ConnectionBuilder<Missing> {
    pub fn binding(self, binding: D1Database) -> D1ConnectionBuilder<Present<D1Database>> {
        let Self {
            transaction_manager,
            session,
            binding: Missing,
        } = self;

        D1ConnectionBuilder {
            transaction_manager,
            session,
            binding: Present(binding),
        }
    }
}

impl<'a, D: Required<D1Database>> D1ConnectionBuilder<D> {
    pub fn transaction_manager(mut self, transaction_manager: D1TransactionManager) -> Self {
        self.transaction_manager = transaction_manager;
        self
    }

    pub fn session_option(mut self, session: SessionOptions) -> Self {
        self.session = session;
        self
    }
}

impl D1ConnectionBuilder<Present<D1Database>> {
    pub fn build(self) -> worker::Result<D1Connection> {
        let Self {
            transaction_manager,
            session,
            binding: Present(binding),
        } = self;

        D1Connection::new(binding, session, transaction_manager)
    }
}

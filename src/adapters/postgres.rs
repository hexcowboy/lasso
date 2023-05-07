use super::Adapter;

#[derive(Debug, Default)]
pub struct PostgresAdapter {
    pub connection_url: String,
}

impl Adapter for PostgresAdapter {
    fn new() -> Self {
        Self::default()
    }

    fn connect(&self) {
        println!("Connecting to Postgres at {}", self.connection_url);
    }

    fn handle_event(&self) {
        println!("Handling event");
    }
}

impl PostgresAdapter {
    pub fn with_connection_url(mut self, connection_url: String) -> Self {
        self.connection_url = connection_url;
        self
    }
}

pub trait Adapter {
    fn new() -> Self;

    fn connect(&self);

    fn handle_event(&self);
}

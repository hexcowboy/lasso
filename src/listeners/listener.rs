use std::sync::Arc;

use crate::adapters::Adapter;

#[derive(Default, Clone)]
pub(crate) struct NoAdapter;
#[derive(Default, Clone)]
pub struct SomeAdapter<T: Adapter>(pub Arc<T>);

pub mod default;

pub use default::{DefaultRenderer, RenderedMessage};

use anyhow::Result;

use crate::event::EventEnvelope;
use crate::router::ResolvedDelivery;

pub trait Renderer: Send + Sync {
    fn render(&self, event: &EventEnvelope, delivery: &ResolvedDelivery)
    -> Result<RenderedMessage>;
}

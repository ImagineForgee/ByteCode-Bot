use std::pin::Pin;
use futures::future;
use poise::FrameworkContext;
use serenity::all::{Context, FullEvent};
use crate::{Data, FullError};

mod on_ready;
mod on_message;

pub struct EventHandlerRegistration {
    pub handler: for<'a> fn(
        ctx: &'a Context,
        event: &'a FullEvent,
        fw: FrameworkContext<'a, Data, FullError>,
        data: &'a Data,
    ) -> Pin<Box<dyn Future<Output = Result<(), FullError>> + Send + 'a>>,
}
inventory::collect!(EventHandlerRegistration);

pub fn dispatch_all_events<'a>(
    ctx: &'a Context,
    event: &'a FullEvent,
    fw: FrameworkContext<'a, Data, FullError>,
    data: &'a Data,
) -> Pin<Box<dyn Future<Output = Result<(), FullError>> + Send + 'a>> {
    let futures = inventory::iter::<EventHandlerRegistration>()
        .map(move |reg| (reg.handler)(ctx, event, fw.clone(), data));

    Box::pin(async move {
        future::try_join_all(futures).await?;
        Ok(())
    })
}
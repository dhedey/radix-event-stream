use ::event_handler::event_handler;
use log::info;
use radix_engine_common::ScryptoSbor;
use radix_event_stream::error::{EventHandlerError, TransactionHandlerError};
use radix_event_stream::event_handler::{EventHandlerContext, HandlerRegistry};
use radix_event_stream::processor::SimpleTransactionStreamProcessor;
use radix_event_stream::sources::gateway::GatewayTransactionStream;
use scrypto::prelude::*;
use std::env;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Clone)]
struct AppState {
    number: u64,
}

#[derive(ScryptoSbor, Debug)]
pub struct InstantiateEvent {
    x_address: ResourceAddress,
    y_address: ResourceAddress,
    context_fee_rate: Decimal,
    liquidity_pool_address: ComponentAddress,
    pool_address: ComponentAddress,
}

// #[event_handler]
// pub fn handle_instantiate_event(
//     context: EventHandlerContext<AppState>,
//     event: InstantiateEvent,
// ) -> Result<(), EventHandlerError> {
//     info!(
//         "Handling the {}th instantiate event: {:#?}",
//         context.app_state.number, event
//     );
//     context.app_state.number += 1;
//     Ok(())
// }

// fn handle_instantiate_event<STATE: Clone + Send + Sync>(
//     context: EventHandlerContext<AppState>,
//     event: Vec<u8>,
// ) -> Pin<Box<dyn Future<Output = Result<(), EventHandlerError>> + Send + '_>> {
//     Box::pin(async move {
//         println!(
//             "Handling the {}th instantiate event: {:#?}",
//             context.app_state.number, event
//         );
//         Ok(())
//     })
// }

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    // Create a new handler registry
    let mut handler_registry: HandlerRegistry<AppState> =
        HandlerRegistry::new();

    // Add the instantiate event handler to the registry
    handler_registry.add_handler(
        "package_rdx1p5l6dp3slnh9ycd7gk700czwlck9tujn0zpdnd0efw09n2zdnn0lzx",
        "InstantiateEvent",
        handle_instantiate_event::<AppState>,
    );

    // Create a new transaction stream, which the processor will use
    // as a source of transactions.
    let stream = GatewayTransactionStream::new(
        1919391,
        100,
        "https://mainnet.radixdlt.com".to_string(),
    );

    // Start with parameters.
    SimpleTransactionStreamProcessor::run_with(
        stream,
        handler_registry,
        AppState { number: 1 },
    )
    .await;
}

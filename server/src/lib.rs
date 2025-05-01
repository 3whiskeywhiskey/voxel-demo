use spacetimedb::{reducer, ReducerContext};

pub mod terrain;

#[reducer(init)]
pub fn init(_ctx: &ReducerContext) {
    // Called when the module is initially published
}

#[reducer(client_connected)]
pub fn identity_connected(_ctx: &ReducerContext) {
    // Called everytime a new client connects
}

#[reducer(client_disconnected)]
pub fn identity_disconnected(_ctx: &ReducerContext) {
    // Called everytime a client disconnects
}

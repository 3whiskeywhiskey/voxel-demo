// src/lib.rs

//! Shared terrain domain types for both client & server.
//! Server builds enable the `server` feature to pull in spacetimedb macros.

pub mod coords;
pub mod heightmap;
pub mod density;
pub mod mesh;
pub mod material;
pub mod prelude;
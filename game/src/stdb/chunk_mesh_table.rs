// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN YOUR MODULE SOURCE CODE INSTEAD.

#![allow(unused, clippy::all)]
use super::chunk_mesh_type::ChunkMesh;
use super::xz_coords_type::XzCoords;
use spacetimedb_sdk::__codegen::{self as __sdk, __lib, __sats, __ws};

/// Table handle for the table `chunk_mesh`.
///
/// Obtain a handle from the [`ChunkMeshTableAccess::chunk_mesh`] method on [`super::RemoteTables`],
/// like `ctx.db.chunk_mesh()`.
///
/// Users are encouraged not to explicitly reference this type,
/// but to directly chain method calls,
/// like `ctx.db.chunk_mesh().on_insert(...)`.
pub struct ChunkMeshTableHandle<'ctx> {
    imp: __sdk::TableHandle<ChunkMesh>,
    ctx: std::marker::PhantomData<&'ctx super::RemoteTables>,
}

#[allow(non_camel_case_types)]
/// Extension trait for access to the table `chunk_mesh`.
///
/// Implemented for [`super::RemoteTables`].
pub trait ChunkMeshTableAccess {
    #[allow(non_snake_case)]
    /// Obtain a [`ChunkMeshTableHandle`], which mediates access to the table `chunk_mesh`.
    fn chunk_mesh(&self) -> ChunkMeshTableHandle<'_>;
}

impl ChunkMeshTableAccess for super::RemoteTables {
    fn chunk_mesh(&self) -> ChunkMeshTableHandle<'_> {
        ChunkMeshTableHandle {
            imp: self.imp.get_table::<ChunkMesh>("chunk_mesh"),
            ctx: std::marker::PhantomData,
        }
    }
}

pub struct ChunkMeshInsertCallbackId(__sdk::CallbackId);
pub struct ChunkMeshDeleteCallbackId(__sdk::CallbackId);

impl<'ctx> __sdk::Table for ChunkMeshTableHandle<'ctx> {
    type Row = ChunkMesh;
    type EventContext = super::EventContext;

    fn count(&self) -> u64 {
        self.imp.count()
    }
    fn iter(&self) -> impl Iterator<Item = ChunkMesh> + '_ {
        self.imp.iter()
    }

    type InsertCallbackId = ChunkMeshInsertCallbackId;

    fn on_insert(
        &self,
        callback: impl FnMut(&Self::EventContext, &Self::Row) + Send + 'static,
    ) -> ChunkMeshInsertCallbackId {
        ChunkMeshInsertCallbackId(self.imp.on_insert(Box::new(callback)))
    }

    fn remove_on_insert(&self, callback: ChunkMeshInsertCallbackId) {
        self.imp.remove_on_insert(callback.0)
    }

    type DeleteCallbackId = ChunkMeshDeleteCallbackId;

    fn on_delete(
        &self,
        callback: impl FnMut(&Self::EventContext, &Self::Row) + Send + 'static,
    ) -> ChunkMeshDeleteCallbackId {
        ChunkMeshDeleteCallbackId(self.imp.on_delete(Box::new(callback)))
    }

    fn remove_on_delete(&self, callback: ChunkMeshDeleteCallbackId) {
        self.imp.remove_on_delete(callback.0)
    }
}

#[doc(hidden)]
pub(super) fn register_table(client_cache: &mut __sdk::ClientCache<super::RemoteModule>) {
    let _table = client_cache.get_or_make_table::<ChunkMesh>("chunk_mesh");
}
pub struct ChunkMeshUpdateCallbackId(__sdk::CallbackId);

impl<'ctx> __sdk::TableWithPrimaryKey for ChunkMeshTableHandle<'ctx> {
    type UpdateCallbackId = ChunkMeshUpdateCallbackId;

    fn on_update(
        &self,
        callback: impl FnMut(&Self::EventContext, &Self::Row, &Self::Row) + Send + 'static,
    ) -> ChunkMeshUpdateCallbackId {
        ChunkMeshUpdateCallbackId(self.imp.on_update(Box::new(callback)))
    }

    fn remove_on_update(&self, callback: ChunkMeshUpdateCallbackId) {
        self.imp.remove_on_update(callback.0)
    }
}

#[doc(hidden)]
pub(super) fn parse_table_update(
    raw_updates: __ws::TableUpdate<__ws::BsatnFormat>,
) -> __sdk::Result<__sdk::TableUpdate<ChunkMesh>> {
    __sdk::TableUpdate::parse_table_update(raw_updates).map_err(|e| {
        __sdk::InternalError::failed_parse("TableUpdate<ChunkMesh>", "TableUpdate")
            .with_cause(e)
            .into()
    })
}

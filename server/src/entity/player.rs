// src/entity/player.rs


pub struct StdbPlayer {
    #[primary_key]
    pub id: u64,
    pub name: String,
    pub transform: StdbTransform,
}

#[reducer]
pub fn on_player_created(ctx: &ReducerContext, player: StdbPlayer) -> Result<(), String> {
    ctx.db.player().insert(player);
    Ok(())
}

#[reducer]
pub fn on_player_updated(ctx: &ReducerContext, player: StdbPlayer) -> Result<(), String> {
    ctx.db.player().update(player);
    Ok(())
}

#[reducer]
pub fn on_player_deleted(ctx: &ReducerContext, player: StdbPlayer) -> Result<(), String> {
    ctx.db.player().delete(player);
    Ok(())
}






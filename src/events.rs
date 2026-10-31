use bevy::prelude::*;

// Ця подія генерується локально мишкою, АБО приходить з мережі від сервера.
// Логіка гри реагує ТІЛЬКИ на події, а не на мишку!
#[derive(Message, Debug, Clone)]
pub struct CommandMoveUnit {
    pub entity: Entity, // В мультиплеєрі тут буде глобальний NetworkId юніта
    pub target_x: i32,
    pub target_z: i32,
}
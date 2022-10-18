use async_graphql::*;

#[derive(Default)]
pub struct MixerQuery;

#[Object]
impl MixerQuery {
    async fn get_volume(&self, ctx: &Context<'_>) -> i32 {
        0
    }
}

#[derive(Default)]
pub struct MixerMutation;

#[Object]
impl MixerMutation {
    async fn set_volume(&self, ctx: &Context<'_>, volume: i32) -> bool {
        false
    }

    async fn set_mute(&self, ctx: &Context<'_>, mute: bool) -> bool {
        false
    }
}

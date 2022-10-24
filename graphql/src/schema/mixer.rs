use async_graphql::*;

#[derive(Default)]
pub struct MixerQuery;

#[Object]
impl MixerQuery {
    async fn get_volume(&self, ctx: &Context<'_>) -> Result<i32, Error> {
        todo!()
    }
}

#[derive(Default)]
pub struct MixerMutation;

#[Object]
impl MixerMutation {
    async fn set_volume(&self, ctx: &Context<'_>, volume: i32) -> Result<bool, Error> {
        todo!()
    }

    async fn set_mute(&self, ctx: &Context<'_>, mute: bool) -> Result<bool, Error> {
        todo!()
    }
}

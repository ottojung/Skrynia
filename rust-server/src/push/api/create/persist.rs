use crate::{Result, State, Subscription};
pub fn persist(state: &State, sub: &Subscription) -> Result<()> {
    state.persist_json(
        &state
            .dir
            .join("state/push-subs")
            .join(format!("{}.json", sub.id)),
        sub,
    )
}

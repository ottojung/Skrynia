use crate::{Result, State, Subscription, write_json};
pub fn persist(state: &State, sub: &Subscription) -> Result<()> {
    write_json(
        &state
            .dir
            .join("state/push-subs")
            .join(format!("{}.json", sub.id)),
        sub,
    )
    .map_err(|_| crate::internal())
}

mod dedup;
mod persist;
mod register;
use crate::{PushInput, Result, State};

pub use dedup::same_events;
pub use persist::persist;
impl State {
    pub fn subscribe(
        &self,
        input: PushInput,
        cap: Option<&str>,
        token: Option<&str>,
    ) -> Result<serde_json::Value> {
        let mut data = self.data.lock().map_err(|_| crate::internal())?;
        self.require_healthy()?;
        let existing = data
            .push
            .subscriptions
            .values()
            .find(|s| {
                s.endpoint == input.endpoint
                    && s.locations == input.locations
                    && same_events(&s.events, &input.events)
            })
            .cloned();
        if let Some(mut sub) = existing {
            let capability = dedup::rotate(self, &mut sub, &input, cap)?;
            data.push.put_sub(sub.clone());
            return Ok(serde_json::json!({"id":sub.id,"capability":capability,"deduped":true}));
        }
        register::create(self, &mut data, input, token)
    }
}

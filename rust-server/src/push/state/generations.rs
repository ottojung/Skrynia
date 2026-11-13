use crate::{Notification, Subscription};
use std::io::Read;
use std::{collections::HashMap, io, path::Path};
pub fn restore(
    root: &Path,
    subscriptions: &HashMap<String, Subscription>,
    outbox: &mut HashMap<String, Notification>,
) -> io::Result<()> {
    let file = root.join("state/push/outbox-generations.json");
    let mut reader = match crate::open_regular(&file) {
        Ok(reader) => reader,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    let mut contents = Vec::new();
    reader.read_to_end(&mut contents)?;
    let generations: HashMap<String, u64> = serde_json::from_slice(&contents)?;
    for (id, generation) in generations {
        if !crate::valid_identifier(&id) || generation == 0 {
            return Err(io::Error::other("invalid outbox generation record"));
        }
        if let Some(pending) = outbox.get_mut(&id) {
            pending.generation = pending.generation.max(generation);
        } else if subscriptions.contains_key(&id) {
            outbox.insert(
                id.clone(),
                Notification {
                    subscription_id: id,
                    generation,
                    attempts: 0,
                    next_at: 0,
                },
            );
        }
    }
    Ok(())
}

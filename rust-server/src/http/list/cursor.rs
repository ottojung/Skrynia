use super::params::Params;
use crate::Snapshot;
pub fn entries(state: &Snapshot, request: &Params) -> Vec<String> {
    let Some(prefix) = request.locations.strip_suffix('*') else {
        return if state.objects.contains_key(&request.locations)
            && request
                .after
                .as_ref()
                .is_none_or(|after| after < &request.locations)
        {
            vec![request.locations.clone()]
        } else {
            Vec::new()
        };
    };
    let start = request
        .after
        .as_deref()
        .filter(|after| *after > prefix)
        .unwrap_or(prefix);
    let mut output = Vec::new();
    for (key, _) in state.objects.range(start.to_string()..) {
        if !key.starts_with(prefix) {
            break;
        }
        if request.after.as_ref().is_some_and(|after| key <= after) {
            continue;
        }
        output.push(key.clone());
        if output.len() > request.limit {
            break;
        }
    }
    output
}
#[cfg(test)]
mod tests;

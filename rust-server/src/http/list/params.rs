use crate::{Result, bad, valid_key, valid_scope};

pub struct Params {
    pub locations: String,
    pub limit: usize,
    pub after: Option<String>,
}
pub fn parse(query: &str) -> Result<Params> {
    let params = form_urlencoded::parse(query.as_bytes())
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect::<Vec<_>>();
    if !params.iter().any(|(k, _)| k == "locations")
        || params
            .iter()
            .any(|(k, _)| !["locations", "limit", "after"].contains(&k.as_str()))
        || ["locations", "limit", "after"]
            .iter()
            .any(|k| params.iter().filter(|(n, _)| n == k).count() > 1)
    {
        return Err(bad("invalid_locations"));
    }
    let param = |name| {
        params
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    };
    let locations = param("locations").unwrap();
    if !valid_scope(locations) {
        return Err(bad("invalid_locations"));
    }
    let limit = param("limit").map_or(Ok(1000usize), |n| {
        n.parse().map_err(|_| bad("invalid_limit"))
    })?;
    if !(1..=1000).contains(&limit) {
        return Err(bad("invalid_limit"));
    }
    let after = param("after");
    if after.is_some_and(|n| !valid_key(n)) {
        return Err(bad("invalid_cursor"));
    }
    Ok(Params {
        locations: locations.into(),
        limit,
        after: after.map(str::to_string),
    })
}

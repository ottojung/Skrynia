use crate::Signature;
use std::collections::BTreeMap;

fn letter(name: &str) -> Result<char, String> {
    match name {
        "i32" => Ok('i'),
        "i64" => Ok('l'),
        _ => Err(format!("invalid integer type: {name:?}")),
    }
}

fn notation(value: &serde_json::Value) -> Result<String, String> {
    let list = value.as_array().ok_or("signature types must be an array")?;
    if list.len() > 16 {
        return Err("signature too long".into());
    }
    list.iter()
        .map(|v| {
            let name = v
                .as_str()
                .ok_or_else(|| "type must be a string".to_string())?;
            letter(name)
        })
        .collect()
}

pub fn from_discovery(value: &serde_json::Value) -> Result<BTreeMap<String, Signature>, String> {
    let list = value.as_array().ok_or("syscalls must be an array")?;
    if list.len() > 256 {
        return Err("syscall list too long".into());
    }
    let mut result = BTreeMap::new();
    for item in list {
        let name = item["name"].as_str().ok_or("missing syscall name")?;
        crate::sys::extensions::check_name(name)?;
        let params = notation(&item["params"])?;
        let results = notation(&item["results"])?;
        let signature = Signature::new(&params, &results)?;
        if result.insert(name.into(), signature).is_some() {
            return Err(format!("duplicate syscall: {name}"));
        }
    }
    Ok(result)
}

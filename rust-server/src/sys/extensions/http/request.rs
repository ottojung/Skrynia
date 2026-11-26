use std::io::Read;

const MAX_BYTES: u64 = 4 * 1024;

pub(super) fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .no_proxy()
        .build()
        .map_err(|e| e.to_string())
}

pub(super) fn call(
    client: &reqwest::blocking::Client,
    url: &reqwest::Url,
    signature: &crate::Signature,
    args: &[wasmtime::Val],
) -> Result<Vec<wasmtime::Val>, String> {
    let body = crate::sys::signature::encode(signature, args)?;
    let response = client
        .post(url.clone())
        .header("Content-Type", "application/octet-stream")
        .body(body)
        .send()
        .map_err(|e| e.to_string())?;
    if response.status() != reqwest::StatusCode::OK {
        return Err(format!("syscall provider status: {}", response.status()));
    }
    let mut bytes = Vec::new();
    response
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("syscall response too large".into());
    }
    crate::sys::signature::decode(signature, &bytes)
}

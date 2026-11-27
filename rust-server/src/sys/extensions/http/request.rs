use std::io::Read;

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
    let expected = crate::sys::signature::frame_len(signature.results())?;
    let limit = expected.checked_add(1).ok_or("reply length overflow")?;
    let mut bytes = Vec::with_capacity(expected);
    response
        .take(limit as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() != expected {
        return Err("syscall response length mismatch".into());
    }
    crate::sys::signature::decode(signature, &bytes)
}

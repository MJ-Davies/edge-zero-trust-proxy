use worker::*;
use crate::utils::ai::*;
use crate::utils::auth::*;

pub async fn _req_debug(req: Request) -> Result<String> {
    let mut out = String::new();

    out.push_str("========= HEADERS =========\n");
    for (key, value) in req.headers().entries() {
        out.push_str(&format!("{}: {}\n", key, value));
    }

    out.push_str("========= URL =========\n");
    out.push_str(&format!("URL: {}\n", req.url()?));

    Ok(out)
}

pub async fn reroute_req(mut req: Request, state: RouteContext<()>) -> Result<Response> {
    // Authorization (remove if not using Cloudflare Zero Trust)
    match verify_identity(&req, &state.env).await {
        Ok(email) => email,
        Err(_) => return Response::error("Unauthorized: Zero Trust Perimeter Violation", 401),
    };
    
    let method = req.method();

    let body_bytes = match method {
        Method::Get | Method::Head => None,
        _ => Some(req.bytes().await?),
    };

    if let Some(bytes) = &body_bytes {
        let payload_str = String::from_utf8_lossy(bytes);
        let verdict = analyze_payload(&state.env, &payload_str).await?;

        if verdict.contains("UNSAFE") {
            return Response::error("Payload blocked by Edge AI", 403);
        }
    }

    let upstream = state.env.var("UPSTREAM_URL")?.to_string(); 
    let upstream_url = format!("{}{}", upstream, req.path());

    let headers = req.headers().clone();
    headers.set("X-Proxy", "EdgeZeroTrust")?; 
    headers.delete("Host")?;

    let init = RequestInit {
        method,
        headers,
        body: body_bytes.map(|b| b.into()), 
        cf: CfProperties::default(),
        ..Default::default()
    };

    let outbound_req = Request::new_with_init(upstream_url.as_str(), &init)?;
    
    Fetch::Request(outbound_req).send().await
}

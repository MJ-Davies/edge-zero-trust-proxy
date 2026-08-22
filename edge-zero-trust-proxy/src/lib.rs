use worker::*;

#[event(start)]
fn start_init() {
    console_error_panic_hook::set_once();
}

async fn req_debug(req: Request) -> Result<String> {
    let mut out = String::new();

    out.push_str("========= HEADERS =========\n");
    for (key, value) in req.headers().entries() {
        out.push_str(&format!("{}: {}\n", key, value));
    }

    out.push_str("========= URL =========\n");
    out.push_str(&format!("URL: {}\n", req.url()?));

    Ok(out)

}

async fn reroute_req(req: Request, _state: RouteContext<()>) -> Result<Response> {
    let upstream = "https://example.com"; // Temporary backend domain
    let upstream_url = format!("{}{}", upstream, req.path());

    let headers = req.headers().clone();
    headers.set("X-Proxy", "EdgeZeroTrust")?; // Temporary header rename
    headers.delete("Host")?;

    let method = req.method();
    let body = match method {
        Method::Get | Method::Head => None,
        _ => {
            let mut cloned = req.clone()?;
            let bytes = cloned.bytes().await?;
            Some(bytes.into())
        },
    }; 

    let init = RequestInit {
        method,
        headers,
        body,
        cf: CfProperties::default(),
        ..Default::default()
    };

    let outbound_req = Request::new_with_init(upstream_url.as_str(), &init)?;
    
    // Debug print of computed request header and url
    Response::ok(req_debug(outbound_req).await?)
}

#[event(fetch)]
async fn fetch(
    req: Request,
    env: Env,
    _ctx: Context,
) -> Result<Response> {
    Router::new()
        .get("/", |_, _| {
            Response::ok("Hello World!")
        })
        .get("/health", |_, _| {
            Response::ok("Edge zero trust proxy server is running")
        })
        .on_async("/*path", reroute_req)
        .run(req, env)
        .await
}

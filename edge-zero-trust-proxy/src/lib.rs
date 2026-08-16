use worker::*;

#[event(start)]
fn start_init() {
    console_error_panic_hook::set_once();
}

async fn reroute_req(_req: Request, _state: RouteContext<()>) -> Result<Response> {
    // NOOP
    Response::ok("Called reroute_req")  
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

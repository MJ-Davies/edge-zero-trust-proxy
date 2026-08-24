mod proxy;
mod utils;

use worker::*;
use proxy::*;

#[event(start)]
fn start_init() {
    console_error_panic_hook::set_once();
}

#[event(fetch)]
async fn fetch(
    req: Request,
    env: Env,
    _ctx: Context,
) -> Result<Response> {
    Router::new()
        .get("/proxyHealth", |_, _| {
            Response::ok("Edge zero trust proxy server is running")
        })
        .on_async("/*path", reroute_req)
        .on_async("/", reroute_req)
        .run(req, env)
        .await
}

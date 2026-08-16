use worker::*;

#[event(start)]
fn init() {
    console_error_panic_hook::set_once();
}

#[event(fetch)]
async fn fetch(
    _req: Request,
    _env: Env,
    _ctx: Context,
) -> Result<Response> {
    Response::ok("Hello World!")
}

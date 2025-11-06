use worker::*;

pub async fn get_handler(req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let mut url = req.url()?;
    url.set_path("/create");
    Response::redirect(url)
} 
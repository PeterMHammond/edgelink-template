use worker::*;
use super::not_found;

pub async fn get_handler(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    match ctx.env.kv("edgelink")?.get(ctx.param("id").unwrap()).text().await? {
        Some(url) => Response::redirect(Url::parse(&url)?),
        None => not_found::handler(req, ctx).await,
    }
}

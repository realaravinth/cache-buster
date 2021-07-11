use std::borrow::Cow;

use actix_web::body::Body;
use actix_web::{get, http::header, web, HttpResponse, Responder};
use actix_web::{App, HttpServer};
use lazy_static::lazy_static;
use log::info;
use mime_guess::from_path;
use rust_embed::RustEmbed;

use cache_buster::Files;

mod index;

/// 1. Set a riddicolusly high cache age
pub const CACHE_AGE: u32 = 60 * 60 * 24 * 365;

lazy_static! {
    /// 2. create filemap
    pub static ref FILES: Files = {
        let map = include_str!("./cache_buster_data.json");
        Files::new(&map)
    };
    pub static ref INDEX: String = index::get_index();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");

    let ip = "localhost:2080";

    pretty_env_logger::init();

    info!("Starting server at http://{}", &ip);
    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .configure(services)
    })
    .bind(ip)
    .unwrap()
    .run()
    .await?;
    Ok(())
}

/// 3. Embed files. Or not. You can also read files dynamically
#[derive(RustEmbed)]
#[folder = "dist/"]
struct Asset;

fn handle_assets(path: &str) -> HttpResponse {
    match Asset::get(path) {
        Some(content) => {
            let body: Body = match content {
                Cow::Borrowed(bytes) => bytes.into(),
                Cow::Owned(bytes) => bytes.into(),
            };

            HttpResponse::Ok()
                // 3. Set proper cache-control headers with cache age set from step 1
                .insert_header(header::CacheControl(vec![
                    header::CacheDirective::Public,
                    header::CacheDirective::Extension("immutable".into(), None),
                    header::CacheDirective::MaxAge(CACHE_AGE),
                ]))
                .content_type(from_path(path).first_or_octet_stream().as_ref())
                .body(body)
        }
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}

#[get("/dist/{_:.*}")]
pub async fn static_files(path: web::Path<String>) -> impl Responder {
    info!("fetching file: {}", &path);
    handle_assets(&path)
}

#[get("/")]
pub async fn serve_index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(&*INDEX)
}

#[derive(RustEmbed)]
#[folder = "static/no-cache/"]
struct Favicons;

fn handle_favicons(path: &str) -> HttpResponse {
    match Favicons::get(path) {
        Some(content) => {
            let body: Body = match content {
                Cow::Borrowed(bytes) => bytes.into(),
                Cow::Owned(bytes) => bytes.into(),
            };

            HttpResponse::Ok()
                .insert_header(header::CacheControl(vec![
                    header::CacheDirective::Public,
                    header::CacheDirective::Extension("immutable".into(), None),
                    header::CacheDirective::MaxAge(CACHE_AGE),
                ]))
                .content_type(from_path(path).first_or_octet_stream().as_ref())
                .body(body)
        }
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}

#[get("/{file}")]
pub async fn favicons(path: web::Path<String>) -> impl Responder {
    handle_favicons(&path)
}

fn services(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(static_files);
    cfg.service(serve_index);
    cfg.service(favicons);
}

#[cfg(test)]
mod tests {
    use actix_web::http::StatusCode;
    use actix_web::test;

    use super::*;

    #[actix_rt::test]
    async fn static_assets_work() {
        let app = test::init_service(App::new().configure(services)).await;

        let img_resp = test::call_service(
            &app,
            test::TestRequest::get()
                .uri(
                    &crate::FILES
                        .get_full_path("./static/cachable/img/Spock_vulcan-salute.png")
                        .unwrap()[1..],
                )
                .to_request(),
        )
        .await;
        assert_eq!(img_resp.status(), StatusCode::OK);

        let css_resp = test::call_service(
            &app,
            test::TestRequest::get()
                .uri(
                    &crate::FILES
                        .get_full_path("./static/cachable/img/Spock_vulcan-salute.png")
                        .unwrap()[1..],
                )
                .to_request(),
        )
        .await;
        assert_eq!(css_resp.status(), StatusCode::OK);

        let favicon_resp = test::call_service(
            &app,
            test::TestRequest::get().uri("/favicon.ico").to_request(),
        )
        .await;
        assert_eq!(favicon_resp.status(), StatusCode::OK);
    }
}

use actix_web::web;
use crate::handlers::upload;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/upload")
            .route(web::post().to(upload))
    );
}
use actix_web::web;
use crate::handlers::{upload, events_stream};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/upload")
            .route(web::post().to(upload))
    )
    .service(
        web::resource("/events")
            .route(web::get().to(events_stream))
    );
}
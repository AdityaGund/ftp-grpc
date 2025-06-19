use actix_web::web;
// use crate::handlers::{upload, events_stream};
use crate::handlers::upload;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(upload);
    // .service(
    //     web::resource("/events")
    //         .route(web::get().to(events_stream))
    // );
}
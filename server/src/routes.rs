use actix_web::web;
use crate::handlers::{add_user, available_banks};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    // cfg.service(login);
    cfg.service(add_user);
    cfg.service(available_banks);
}

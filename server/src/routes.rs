use actix_web::web;
use crate::handlers::{login, add_user};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(login);
    cfg.service(add_user);
}

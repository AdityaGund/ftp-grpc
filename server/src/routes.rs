use actix_web::web;
use crate::handlers::{add_user, available_banks, update_user, delete_user, list_users, fetch_file_info, fetch_received_file_info};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    // cfg.service(login);
    cfg.service(add_user);
    cfg.service(update_user);
    cfg.service(delete_user);
    cfg.service(available_banks);
    cfg.service(list_users);
    cfg.service(fetch_file_info);
    cfg.service(fetch_received_file_info);
}

use std::{net, sync::Arc};
use actix_cors::Cors;
use actix_web::{middleware::Logger, web::{self, Data}, App, HttpServer};
use futures::lock::Mutex;

use crate::ws::controllers::{admin, images};
use crate::{cache::CacheDriver, models::Image, storage::Storage};

pub struct Config { pub adminkey: Option<String> }

pub async fn run(
    address: impl net::ToSocketAddrs, adminkey: Option<String>, origin: Option<String>,
    max_storage_bytes: u64, storage: Arc<Storage>, cache: Arc<CacheDriver<Image>>,
) -> std::io::Result<()> {
    let upload_lock = Arc::new(Mutex::new(()));
    HttpServer::new(move || {
        let mut cors = Cors::default().allow_any_header().allow_any_method().max_age(3600);
        if let Some(org) = origin.clone() { cors = cors.allowed_origin(org.as_str()); }
        else { cors = cors.send_wildcard(); }
        let cfg = Config { adminkey: adminkey.clone() };
        App::new()
            .service(admin::get_public_site)
            .service(web::scope("/images").configure(images::configure))
            .service(web::scope("/admin").configure(admin::configure))
            .wrap(Logger::default()).wrap(cors)
            .app_data(Data::from(cache.clone())).app_data(Data::from(storage.clone()))
            .app_data(Data::new(cfg)).app_data(Data::new(max_storage_bytes))
            .app_data(Data::new(upload_lock.clone()))
    }).bind(address)?.run().await
}
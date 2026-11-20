use crate::cache::CacheDriver;
use crate::images;
use crate::models::{DimensionsOpt, Image};
use crate::storage::Storage;
use crate::ws::util;
use actix_web::web::{Data, ServiceConfig};
use actix_web::{get, web, Error, HttpResponse};
use file_format::FileFormat;
use serde::Deserialize;
use std::io;

#[derive(Deserialize)]
struct ImageListQuery { gallery: Option<String> }

#[get("")]
async fn get_meta_list(
    storage: Data<Storage>, cache: Data<CacheDriver<Image>>, query: web::Query<ImageListQuery>,
) -> Result<HttpResponse, Error> {
    let mut res = images::list(storage.into_inner(), cache.into_inner()).map_err(actix_web::error::ErrorInternalServerError)?;
    if let Some(gallery) = &query.gallery {
        res.retain(|image| image.group.as_deref() == Some(gallery.as_str()));
    }
    Ok(HttpResponse::Ok().json(res))
}

#[get("/{id}/meta")]
async fn get_meta(cache: Data<CacheDriver<Image>>, id: web::Path<String>) -> Result<HttpResponse, Error> {
    let Some(res) = images::cached_details(&cache, id.as_str()).map_err(util::map_err)? else {
        return Ok(HttpResponse::NotFound().finish());
    };
    Ok(HttpResponse::Ok().json(res))
}

#[get("/{id}")]
async fn get_image(storage: Data<Storage>, id: web::Path<String>) -> Result<HttpResponse, Error> {
    let Some(mut res) = images::data(&storage, id.as_str()).map_err(util::map_err)? else {
        return Ok(HttpResponse::NotFound().finish());
    };
    let mut v = Vec::<u8>::new();
    io::copy(&mut res, &mut v)?;
    let format = FileFormat::from_bytes(&v);
    Ok(HttpResponse::Ok().append_header(("Cache-Control", "public, max-age=604800, immutable"))
        .append_header(("Content-Type", format.media_type())).body(v))
}

#[get("/{id}/thumbnail")]
async fn get_image_thumbnail(
    storage: Data<Storage>, id: web::Path<String>, web::Query(dimensions): web::Query<DimensionsOpt>,
) -> Result<HttpResponse, Error> {
    let (width, height) = (dimensions.width.unwrap_or(0), dimensions.height.unwrap_or(0));
    let Some(mut res) = images::thumbnail(&storage, id.as_str(), width, height).map_err(util::map_err)? else {
        return Ok(HttpResponse::NotFound().finish());
    };
    let mut v = Vec::<u8>::new();
    io::copy(&mut res, &mut v)?;
    let format = FileFormat::from_bytes(&v);
    Ok(HttpResponse::Ok().append_header(("Cache-Control", "public, max-age=604800, immutable"))
        .append_header(("Content-Type", format.media_type())).body(v))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(get_meta_list).service(get_meta).service(get_image).service(get_image_thumbnail);
}
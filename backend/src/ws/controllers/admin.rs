use crate::{
    cache::CacheDriver, images, models::Image, site::{self, Gallery, SiteSettings},
    storage::Storage, ws::{extractors::token::Token, server::Config},
};
use actix_web::{
    error::{ErrorBadRequest, ErrorInternalServerError, ErrorPayloadTooLarge, ErrorUnauthorized},
    get, post, put, web::{self, Data, ServiceConfig}, Error, HttpResponse,
};
use chrono::Utc;
use futures::lock::Mutex;
use futures_util::StreamExt;
use serde::Deserialize;
use std::{fs::{self, File}, io::Write, path::Path, sync::Arc};

#[derive(Deserialize)]
struct SettingsInput {
    title: String, tagline: String, about: String, contact_email: String, contact_phone: String,
    theme: String, website_url: String, instagram_url: String, youtube_url: String,
    facebook_url: String, tiktok_url: String,
}
#[derive(Deserialize)]
struct GalleryInput { slug: String, title: String, description: String }
#[derive(Deserialize)]
struct UploadQuery { gallery: String, filename: String }

#[get("/site")]
pub async fn get_public_site(storage: Data<Storage>) -> Result<HttpResponse, Error> {
    let settings = SiteSettings::load(&storage).map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(settings.public_view()))
}

#[get("/site")]
async fn get_admin_site(cfg: Data<Config>, storage: Data<Storage>, token: Token) -> Result<HttpResponse, Error> {
    check_token(&cfg, &token)?;
    Ok(HttpResponse::Ok().json(SiteSettings::load(&storage).map_err(ErrorInternalServerError)?))
}

#[put("/site")]
async fn update_site(
    cfg: Data<Config>, storage: Data<Storage>, token: Token, input: web::Json<SettingsInput>,
) -> Result<HttpResponse, Error> {
    check_token(&cfg, &token)?;
    let mut settings = SiteSettings::load(&storage).map_err(ErrorInternalServerError)?;
    settings.title = input.title.trim().to_string();
    settings.tagline = input.tagline.trim().to_string();
    settings.about = input.about.trim().to_string();
    settings.contact_email = input.contact_email.trim().to_string();
    settings.contact_phone = input.contact_phone.trim().to_string();
    settings.theme = match input.theme.as_str() { "dark" | "warm" | "editorial" => input.theme.clone(), _ => "classic".into() };
    settings.website_url = input.website_url.trim().to_string();
    settings.instagram_url = input.instagram_url.trim().to_string();
    settings.youtube_url = input.youtube_url.trim().to_string();
    settings.facebook_url = input.facebook_url.trim().to_string();
    settings.tiktok_url = input.tiktok_url.trim().to_string();
    settings.save(&storage).map_err(ErrorInternalServerError)?;
    site::record_activity(&storage, "Website updated", "Changed website details or appearance").map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(settings.public_view()))
}

#[get("/dashboard")]
async fn dashboard(
    cfg: Data<Config>, storage: Data<Storage>, cache: Data<CacheDriver<Image>>,
    max_storage_bytes: Data<u64>, token: Token,
) -> Result<HttpResponse, Error> {
    check_token(&cfg, &token)?;
    let settings = SiteSettings::load(&storage).map_err(ErrorInternalServerError)?;
    let photo_count = images::list(storage.clone().into_inner(), cache.into_inner())
        .map_err(ErrorInternalServerError)?.len();
    let used_bytes = storage.content_size().map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "photo_count": photo_count, "gallery_count": settings.galleries.len(),
        "used_bytes": used_bytes, "limit_bytes": *max_storage_bytes, "activity": settings.activity
    })))
}

#[post("/galleries")]
async fn create_gallery(
    cfg: Data<Config>, storage: Data<Storage>, token: Token, input: web::Json<GalleryInput>,
) -> Result<HttpResponse, Error> {
    check_token(&cfg, &token)?;
    let slug = input.slug.trim();
    let title = input.title.trim();
    if !site::is_safe_gallery_slug(slug) || title.is_empty() || title.len() > 100 {
        return Err(ErrorBadRequest("invalid gallery name or slug"));
    }
    fs::create_dir_all(storage.bucket_path("content").join(slug)).map_err(ErrorInternalServerError)?;
    let mut settings = SiteSettings::load(&storage).map_err(ErrorInternalServerError)?;
    let gallery = Gallery { slug: slug.into(), title: title.into(), description: input.description.trim().into() };
    if let Some(existing) = settings.galleries.iter_mut().find(|g| g.slug == slug) { *existing = gallery.clone(); }
    else { settings.galleries.push(gallery.clone()); }
    settings.save(&storage).map_err(ErrorInternalServerError)?;
    site::record_activity(&storage, "Gallery saved", title).map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Created().json(gallery))
}

#[post("/upload")]
async fn upload_image(
    cfg: Data<Config>, storage: Data<Storage>, cache: Data<CacheDriver<Image>>,
    max_storage_bytes: Data<u64>, upload_lock: Data<Arc<Mutex<()>>>, token: Token,
    query: web::Query<UploadQuery>, mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
    check_token(&cfg, &token)?;
    if !site::is_safe_gallery_slug(&query.gallery) { return Err(ErrorBadRequest("invalid gallery")); }
    if !images::is_image(Path::new(&query.filename)) { return Err(ErrorBadRequest("unsupported image type")); }
    let settings = SiteSettings::load(&storage).map_err(ErrorInternalServerError)?;
    if !settings.galleries.iter().any(|g| g.slug == query.gallery) { return Err(ErrorBadRequest("gallery does not exist")); }

    // Serializing uploads prevents concurrent requests from overshooting the total site quota.
    let _quota_guard = upload_lock.lock().await;
    let used_before = storage.content_size().map_err(ErrorInternalServerError)?;
    if used_before >= *max_storage_bytes { return Err(ErrorPayloadTooLarge("storage limit reached")); }

    let original_name = Path::new(&query.filename).file_name().and_then(|s| s.to_str()).unwrap_or("photo");
    let safe_name: String = original_name.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
        .take(100).collect();
    let image_id = format!("{}-{}", Utc::now().timestamp_micros(), safe_name);
    let gallery_dir = storage.bucket_path("content").join(&query.gallery);
    fs::create_dir_all(&gallery_dir).map_err(ErrorInternalServerError)?;
    let file_path = gallery_dir.join(&image_id);
    let mut file = File::create(&file_path).map_err(ErrorInternalServerError)?;
    let mut received = 0u64;

    while let Some(chunk) = payload.next().await {
        let chunk = chunk.map_err(ErrorBadRequest)?;
        received = received.saturating_add(chunk.len() as u64);
        if used_before.saturating_add(received) > *max_storage_bytes {
            drop(file);
            let _ = fs::remove_file(&file_path);
            return Err(ErrorPayloadTooLarge("100 GB storage limit reached"));
        }
        file.write_all(&chunk).map_err(ErrorInternalServerError)?;
    }
    file.flush().map_err(ErrorInternalServerError)?;
    drop(file);
    if received == 0 {
        let _ = fs::remove_file(&file_path);
        return Err(ErrorBadRequest("empty upload"));
    }

    images::cache_single_image_meta(storage.clone().into_inner(), cache.into_inner(), image_id.clone())
        .map_err(ErrorInternalServerError)?;
    site::record_activity(&storage, "Photo uploaded", &format!("{} to {}", query.filename, query.gallery))
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Created().json(serde_json::json!({ "id": image_id, "gallery": query.gallery, "bytes": received })))
}

#[post("/cache/flush")]
async fn flush_cache(cfg: Data<Config>, cache: Data<CacheDriver<Image>>, token: Token) -> Result<HttpResponse, Error> {
    check_token(&cfg, &token)?;
    cache.flush().map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().finish())
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(get_admin_site).service(update_site).service(dashboard)
        .service(create_gallery).service(upload_image).service(flush_cache);
}
fn check_token(cfg: &Config, token: &Token) -> Result<(), Error> {
    let Some(adminkey) = &cfg.adminkey else { return Err(ErrorBadRequest("no admin API key configured")); };
    let Token::Bearer(token) = token else { return Err(ErrorBadRequest("no admin API key provided in request")); };
    if adminkey != token { return Err(ErrorUnauthorized("unauthorized")); }
    Ok(())
}
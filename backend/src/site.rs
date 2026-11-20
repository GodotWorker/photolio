use crate::storage::Storage;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs::{self, File}, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Gallery {
    pub slug: String,
    pub title: String,
    pub description: String,
}
impl Default for Gallery {
    fn default() -> Self {
        Self { slug: "general".into(), title: "General".into(), description: String::new() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityItem {
    pub at: String,
    pub action: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SiteSettings {
    pub title: String,
    pub tagline: String,
    pub about: String,
    pub contact_email: String,
    pub contact_phone: String,
    pub theme: String,
    pub website_url: String,
    pub instagram_url: String,
    pub youtube_url: String,
    pub facebook_url: String,
    pub tiktok_url: String,
    pub galleries: Vec<Gallery>,
    pub activity: Vec<ActivityItem>,
}
impl Default for SiteSettings {
    fn default() -> Self {
        Self {
            title: "My Photography".into(),
            tagline: "A collection of moments".into(),
            about: "Welcome to my photography gallery.".into(),
            contact_email: String::new(),
            contact_phone: String::new(),
            theme: "classic".into(),
            website_url: String::new(),
            instagram_url: String::new(),
            youtube_url: String::new(),
            facebook_url: String::new(),
            tiktok_url: String::new(),
            galleries: vec![Gallery::default()],
            activity: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicSite {
    pub title: String,
    pub tagline: String,
    pub about: String,
    pub contact_email: String,
    pub contact_phone: String,
    pub theme: String,
    pub website_url: String,
    pub instagram_url: String,
    pub youtube_url: String,
    pub facebook_url: String,
    pub tiktok_url: String,
    pub galleries: Vec<Gallery>,
}
impl SiteSettings {
    pub fn load(storage: &Storage) -> Result<Self> {
        let path = storage.root_path().join("site.json");
        if path.exists() {
            return Ok(serde_json::from_reader(File::open(path)?)?);
        }
        let mut settings = Self::default();
        let content_dir = storage.bucket_path("content");
        let mut existing = Vec::new();
        if content_dir.exists() {
            for entry in fs::read_dir(content_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    existing.push(Gallery { slug: name.clone(), title: name, description: String::new() });
                }
            }
        }
        existing.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        if !existing.is_empty() { settings.galleries = existing; }
        settings.save(storage)?;
        Ok(settings)
    }

    pub fn save(&self, storage: &Storage) -> Result<()> {
        fs::create_dir_all(storage.root_path())?;
        let path = storage.root_path().join("site.json");
        let temp = storage.root_path().join("site.json.tmp");
        serde_json::to_writer_pretty(File::create(&temp)?, self)?;
        fs::rename(temp, path)?;
        Ok(())
    }

    pub fn public_view(&self) -> PublicSite {
        PublicSite {
            title: self.title.clone(), tagline: self.tagline.clone(), about: self.about.clone(),
            contact_email: self.contact_email.clone(), contact_phone: self.contact_phone.clone(),
            theme: self.theme.clone(), website_url: self.website_url.clone(),
            instagram_url: self.instagram_url.clone(), youtube_url: self.youtube_url.clone(),
            facebook_url: self.facebook_url.clone(), tiktok_url: self.tiktok_url.clone(),
            galleries: self.galleries.clone(),
        }
    }
}

pub fn record_activity(storage: &Storage, action: &str, detail: &str) -> Result<()> {
    let mut settings = SiteSettings::load(storage)?;
    settings.activity.insert(0, ActivityItem {
        at: chrono::Utc::now().to_rfc3339(), action: action.to_string(), detail: detail.to_string(),
    });
    settings.activity.truncate(50);
    settings.save(storage)
}

pub fn is_safe_gallery_slug(slug: &str) -> bool {
    !slug.is_empty() && slug.len() <= 80 && !slug.starts_with('.')
        && slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ' ')
        && Path::new(slug).components().count() == 1
}
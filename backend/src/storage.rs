use anyhow::Result;
use std::{
    fs::{self, DirEntry, File, Metadata},
    io::{copy, Cursor, ErrorKind, Read, Seek},
    path::{Path, PathBuf},
};

/// Combined trait for readers that also support seeking.
pub trait ReadSeek: Read + Seek {}
impl ReadSeek for File {}
impl ReadSeek for Cursor<Vec<u8>> {}

/// Local data storage implementation to store, list and retrieve image data.
pub struct Storage { root_dir: PathBuf }

impl Storage {
    pub fn new(root_dir: impl Into<PathBuf>) -> Self { Storage { root_dir: root_dir.into() } }
    pub fn root_path(&self) -> &Path { &self.root_dir }
    pub fn create_bucket_if_not_exists(&self, bucket: &str) -> Result<()> {
        fs::create_dir_all(self.bucket_path(bucket))?;
        Ok(())
    }
    pub fn store(&self, bucket: &str, path: &str, reader: &mut dyn Read) -> Result<()> {
        let mut file = File::create(self.bucket_path(bucket).join(path))?;
        copy(reader, &mut file)?;
        Ok(())
    }
    pub fn content_size(&self) -> Result<u64> {
        fn size(path: &Path) -> Result<u64> {
            let mut total = 0u64;
            if !path.exists() { return Ok(0); }
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() { total = total.saturating_add(size(&entry.path())?); }
                else if entry.file_type()?.is_file() { total = total.saturating_add(entry.metadata()?.len()); }
            }
            Ok(total)
        }
        size(&self.bucket_path("content"))
    }
    pub fn read_deep(&self, bucket: &str, name: &str) -> Result<Option<Box<dyn ReadSeek>>> {
        Ok(self.get_object_deep(bucket, name)?.map(|(f, _)| Box::new(f) as Box<dyn ReadSeek>))
    }
    pub fn meta_deep(&self, bucket: &str, name: &str) -> Result<Option<(Metadata, Option<String>)>> {
        self.get_object_deep(bucket, name)?
            .map(|(f, dir)| f.metadata().map(|meta| (meta, dir)))
            .transpose().map_err(Into::into)
    }
    pub fn exists_deep(&self, bucket: &str, name: &str) -> Result<bool> {
        if self.bucket_path(bucket).join(name).exists() { return Ok(true); }
        for dir in self.directories(bucket)? { if dir.path().join(name).exists() { return Ok(true); } }
        Ok(false)
    }
    pub fn list_deep(&self, bucket: &str) -> Result<Vec<(String, Option<String>)>> {
        let mut res = vec![];
        for entry in fs::read_dir(self.bucket_path(bucket))? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if entry.file_type()?.is_dir() {
                for child in fs::read_dir(self.bucket_path(bucket).join(entry.file_name()))? {
                    let child = child?;
                    if child.file_type()?.is_file() { res.push((child.file_name().to_string_lossy().to_string(), Some(name.clone()))); }
                }
            } else if entry.file_type()?.is_file() { res.push((name, None)); }
        }
        Ok(res)
    }
    pub fn bucket_path(&self, bucket: &str) -> PathBuf { self.root_dir.join(bucket) }
    fn directories(&self, bucket: &str) -> Result<Vec<DirEntry>> {
        let res = fs::read_dir(self.bucket_path(bucket))?
            .map(|entry| entry.and_then(|entry| entry.metadata().map(|meta| if meta.is_dir() { Some(entry) } else { None })))
            .filter_map(|v| v.transpose()).collect::<Result<Vec<_>, _>>();
        Ok(res?)
    }
    fn get_object(&self, bucket: &str, dir: impl AsRef<Path>) -> Result<Option<File>> {
        match File::open(self.bucket_path(bucket).join(dir)) {
            Ok(file) => Ok(Some(file)),
            Err(err) if matches!(err.kind(), ErrorKind::NotFound) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
    fn get_object_deep(&self, bucket: &str, name: &str) -> Result<Option<(File, Option<String>)>> {
        match self.get_object(bucket, name)? {
            Some(file) => Ok(Some((file, None))),
            None => {
                for dir in self.directories(bucket)? {
                    if let Some(file) = self.get_object(bucket, Path::new(dir.file_name().as_os_str()).join(name))? {
                        return Ok(Some((file, Some(dir.file_name().to_string_lossy().to_string()))));
                    }
                }
                Ok(None)
            }
        }
    }
}
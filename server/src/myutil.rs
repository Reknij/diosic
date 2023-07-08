use std::{hash::{Hash, Hasher}, collections::hash_map::DefaultHasher, path::{Path}, io::Cursor};
use anyhow::Result;

mod diosic_id;

pub use diosic_id::DiosicID;

pub fn calc_hash<T: Hash + Sized>(obj: &T)-> u64 {
    let mut hasher = DefaultHasher::new();
    obj.hash(&mut hasher);
    hasher.finish()
}

pub fn get_file_name_without_ext<P>(path: P) -> String where P: AsRef<Path> {
    path.as_ref().file_stem().unwrap().to_str().unwrap().to_owned()
}

pub async fn save_binary_file_from_url<P>(url: &str, save_path: P) -> Result<()> where P: AsRef<Path> {
    let response = reqwest::get(url).await?;
    let mut file = std::fs::File::create(save_path)?;
    let mut content =  Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    
    Ok(())
}
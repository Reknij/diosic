use std::{hash::{Hash, Hasher}, collections::hash_map::DefaultHasher, path::{Path}};

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
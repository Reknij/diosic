use std::{
    ffi::{CStr, CString},
    fmt::Debug,
    sync::Arc, path::PathBuf, collections::{HashMap, HashSet},
};

use crate::{config::Config, myutil};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use tokio::{fs, sync::Mutex};
use tracing::{error, info, log::warn};
use walkdir::WalkDir;
use wasmtime::{Engine, Instance, Linker, Module, Store};
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};
use chrono::prelude::*;

const PROCESS_MEDIA_INFO_JSON_FUNCTION: &str = "process_media_info_json";
const GET_PLUGIN_INFO_JSON_FUNCTION: &str = "get_plugin_info_json";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginList {
    pub current: Vec<PluginInfo>,
    pub to_add: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ToAddPlugin {
    pub id: String,
    pub save_path: PathBuf,
    pub url: String,
}

#[derive(Clone)]
pub struct PluginSystem {
    plugins_directory: Option<PathBuf>,
    last_access_config: Arc<Config>,
    _engine: Engine,
    store: Arc<Mutex<Store<WasiCtx>>>,
    instances: Vec<Instance>,
    infos: HashMap<String, (PluginInfo, PathBuf)>,
}

impl Debug for PluginSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginSystem")
            .field("last_access_config", &self.last_access_config)
            .field("instances_length", &self.instances.len())
            .finish()
    }
}

impl PluginSystem {
    pub async fn new(config: Arc<Config>) -> Self {
        let mut wt_config = wasmtime::Config::default();
        wt_config.async_support(true);
        let engine = Engine::new(&wt_config).unwrap();
        let mut linker = Linker::new(&engine);

        wasmtime_wasi::add_to_linker(&mut linker, |s| s).unwrap();
        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_args()
            .unwrap()
            .build();

        let mut store = Store::new(&engine, wasi);

        let mut instances = vec![];
        let mut infos = HashMap::new();

        let mut plugins_directory = None;
        match &config.data_path {
            Some(data_path) => {
                if data_path.exists() {
                    let plugins_path = data_path.join("plugins");
                    if !plugins_path.exists() {
                        fs::create_dir(&plugins_path).await.unwrap();
                    }
                    plugins_directory = Some(plugins_path.clone());
                    let wasm_files = WalkDir::new(plugins_path)
                        .follow_links(false)
                        .into_iter()
                        .filter_map(|file| {
                            file.ok().and_then(|f| {
                                let extension = f
                                    .path()
                                    .extension()
                                    .unwrap_or_default()
                                    .to_str()
                                    .unwrap_or_default();
                                if f.path().is_file() && extension == "wasm" {
                                    return Some(f);
                                }
                                None
                            })
                        });
                    for w in wasm_files {
                        let bytes = tokio::fs::read(w.path()).await;
                        match bytes {
                            Ok(bytes) => {
                                let cpt = Module::new(&engine, &bytes);
                                match cpt {
                                    Ok(cpt) => {
                                        let instance =
                                            linker.instantiate_async(&mut store, &cpt).await;
                                        match instance {
                                            Ok(i) => {
                                                let func = i.get_typed_func::<(), (u32,)>(
                                                    &mut store,
                                                    GET_PLUGIN_INFO_JSON_FUNCTION,
                                                );
                                                let memory = i.get_memory(&mut store, "memory");
                                                let mut info: Option<PluginInfo> = None;
                                                if func.is_ok() && memory.is_some() {
                                                    let f = func.unwrap();
                                                    let memory = memory.unwrap();

                                                    let result = f.call_async(&mut store, ()).await;
                                                    match result {
                                                        Ok((ptr,)) => {
                                                            let info_ptr = unsafe {
                                                                memory
                                                                    .data_ptr(&mut store)
                                                                    .add(ptr as _)
                                                            };
                                                            let info_str = unsafe {
                                                                CStr::from_ptr(info_ptr as _)
                                                                    .to_str()
                                                            };
                                                            if let Ok(str) = info_str {
                                                                info = serde_json::from_str(str).map_err(|err| error!("deserialize plugin info got error: {}", err)).ok()
                                                            }
                                                        }
                                                        Err(err) => error!(
                                                            "call {} got error: {}",
                                                            GET_PLUGIN_INFO_JSON_FUNCTION, err
                                                        ),
                                                    }
                                                } else {
                                                    if let Err(err) = func {
                                                        error!(
                                                            "get {} failed: {}",
                                                            GET_PLUGIN_INFO_JSON_FUNCTION, err
                                                        );
                                                    }

                                                    if memory.is_none() {
                                                        error!("get memory failed");
                                                    }
                                                }

                                                match info {
                                                    Some(info) => {
                                                        info!("Loaded a plugin <{}>", &info.name);

                                                        instances.push(i);
                                                        infos.insert(info.name.clone(), (info, w.into_path()));
                                                    }
                                                    None => todo!(),
                                                }
                                            }
                                            Err(err) => error!("Linker: {}", err),
                                        }
                                    }
                                    Err(err) => {
                                        error!("Component: {}", err)
                                    }
                                }
                            }
                            Err(err) => error!("Can't read the wasm file: {}", err),
                        }
                    }
                } else {
                    warn!("`data_path` is empty so can't load the plugins.")
                }
            }
            None => {
                warn!("`data_path` is empty so can't load the plugins.")
            }
        }
        PluginSystem {
            last_access_config: config.clone(),
            _engine: engine,
            store: Arc::new(Mutex::new(store)),
            instances,
            infos,
            plugins_directory,
        }
    }

    pub async fn scan(&mut self) {
        info!("Scanning all plugin..");
        let newed = PluginSystem::new(self.last_access_config.clone()).await;
        *self = newed;
    }

    pub fn exists_plugins(&self) -> bool {
        self.instances.len() > 0
    }

    pub fn exist_plugin(&self, name: String)-> bool {
        self.infos.contains_key(&name)
    }

    pub async fn process_media_info_json(&self, media_path: &str, media_info: &mut String) {
        let mut cmedia_info = CString::new(media_info.to_owned()).unwrap();
        for i in &self.instances {
            let mut store = &mut *self.store.lock().await;
            let func = i
                .get_typed_func::<(u32, u32), (u32,)>(&mut store, PROCESS_MEDIA_INFO_JSON_FUNCTION);
            let memory = i.get_memory(&mut store, "memory");
            match memory {
                Some(memory) => match func {
                    Ok(f) => {
                        let offset = 0;
                        let bytes = cmedia_info.as_bytes_with_nul();
                        memory.write(&mut *store, offset, bytes).unwrap();
                        let offset2 = offset + bytes.len();
                        memory
                            .write(&mut *store, offset2, media_path.as_bytes())
                            .unwrap();
                        let result = f
                            .call_async(&mut store, (offset as _, (offset2) as _))
                            .await;
                        match result {
                            Ok((ptr,)) => {
                                let modified_ptr =
                                    unsafe { memory.data_ptr(&mut store).add(ptr as _) };
                                let modified = unsafe { CStr::from_ptr(modified_ptr as _) };
                                cmedia_info = modified.to_owned();
                            }
                            Err(err) => error!("{}", err),
                        }
                    }
                    Err(err) => error!("{}", err),
                },
                None => {
                    error!("failed to find `memory` export.")
                }
            }
        }
        *media_info = cmedia_info.to_str().unwrap().to_string();
    }

    pub fn get_plugins_info(&self) -> Vec<PluginInfo> {
        let mut current: Vec<PluginInfo> = Vec::with_capacity(self.infos.len());
        for (_, (info, path)) in &self.infos {
            current.push(info.to_owned());
        }
        current
    }
}

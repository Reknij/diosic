use std::{
    ffi::{CStr, CString},
    fmt::Debug,
    sync::Arc,
};

use crate::config::Config;
use tokio::{fs, sync::Mutex};
use tracing::{error, info, log::warn};
use walkdir::WalkDir;
use wasmtime::{Engine, Instance, Linker, Module, Store};
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};

const PROCESS_MEDIA_INFO_JSON_FUNCTION: &str = "process_media_info_json";

#[derive(Clone)]
pub struct PluginSystem {
    last_access_config: Arc<Config>,
    _engine: Engine,
    store: Arc<Mutex<Store<WasiCtx>>>,
    instances: Vec<Instance>,
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
        match &config.data_path {
            Some(data_path) => {
                if data_path.exists() {
                    let plugins_path = data_path.join("plugins");
                    if !plugins_path.exists() {
                        fs::create_dir(&plugins_path).await.unwrap();
                    }
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
                        let plugin_name = w.path().file_stem().unwrap().to_str().unwrap();
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
                                                instances.push(i);
                                                info!("Loaded a plugin <{}>", plugin_name);
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
        }
    }

    pub fn exists_plugins(&self) -> bool {
        self.instances.len() > 0
    }

    pub async fn process_media_info_json(&self, media_path: &str, media_info: &mut String) {
        let mut cmedia_info = CString::new(media_info.to_owned()).unwrap();
        for i in &self.instances {
            let mut store = &mut *self.store.lock().await;
            let func =
                i.get_typed_func::<(u32, u32), (u32,)>(&mut store, PROCESS_MEDIA_INFO_JSON_FUNCTION);
            let memory = i.get_memory(&mut store, "memory");
            match memory {
                Some(memory) => match func {
                    Ok(f) => {
                        let offset = 0;
                        let bytes = cmedia_info.as_bytes_with_nul();
                        memory
                            .write(&mut *store, offset, bytes)
                            .unwrap();
                        let offset2 = offset + bytes.len();
                        memory
                            .write(&mut *store, offset2, media_path.as_bytes())
                            .unwrap();
                        let result = f.call_async(&mut store, (offset as _, (offset2) as _)).await;
                        match result {
                            Ok((ptr,)) => {
                                let modified_ptr = unsafe {
                                    memory.data_ptr(&mut store).add(ptr as _)
                                };
                                let modified = unsafe {
                                    CStr::from_ptr(modified_ptr as _)
                                };
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
}

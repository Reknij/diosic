use std::{collections::HashMap, path::PathBuf, time};

use anyhow::Context;
use tokio::{fs, sync::mpsc};
use tracing::{error, info, warn};
use wasmtime::{AsContextMut, Caller, Engine, Extern, Instance, Linker, Module, Store};
use wasmtime_wasi::{
    preview1::{self, WasiP1Ctx},
    WasiCtxBuilder,
};

use crate::library_system::model::MediaInfo;

const PROCESS_MEDIA_INFO_JSON_FUNCTION: &str = "process_media_info_json";
pub struct PluginsContext {
    store: Store<WasiP1Ctx>,
    instances: Vec<(String, Instance)>,
    process_media_info_rx: mpsc::Receiver<String>,
}

impl PluginsContext {
    pub async fn new(plugins: &HashMap<String, PathBuf>) -> Self {
        let mut wt_config = wasmtime::Config::new();
        wt_config.async_support(true);
        let engine = Engine::new(&wt_config).expect("Initialize the wasmtime engine failed!");
        let args = std::env::args().skip(1).collect::<Vec<_>>();
        let mut linker: Linker<WasiP1Ctx> = Linker::new(&engine);
        preview1::add_to_linker_async(&mut linker, |t| t)
            .with_context(|| "Wasi preview1 add to linker failed!")
            .unwrap();
        let wasi_ctx: WasiP1Ctx = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_env()
            .args(&args)
            .build_p1();

        let (tx, process_media_info_rx) = mpsc::channel(plugins.len());
        let tx = tx.clone();
        linker
            .func_wrap2_async(
                "env",
                "callback",
                move |mut caller: Caller<'_, WasiP1Ctx>, ptr: u32, len: u32| {
                    let tx = tx.clone();
                    Box::new(async move {
                        let mem = match caller.get_export("memory") {
                            Some(Extern::Memory(mem)) => mem,
                            _ => return error!("failed to find host memory"),
                        };
                        let data = mem
                            .data(&caller)
                            .get(ptr as u32 as usize..)
                            .and_then(|arr| arr.get(..len as u32 as usize));
                        match data {
                            Some(data) => match std::str::from_utf8(data) {
                                Ok(s) => tx
                                    .send(s.to_owned())
                                    .await
                                    .expect("Send modified media info to rx failed!"),
                                Err(_) => return error!("invalid utf-8"),
                            },
                            None => return error!("pointer/length out of bounds"),
                        };
                    })
                },
            )
            .with_context(|| "wrap call back function failed!")
            .unwrap();

        let mut store = Store::new(&engine, wasi_ctx);
        let mut instances = Vec::with_capacity(plugins.len());
        for (name, file_path) in plugins {
            let start = time::Instant::now();
            let module_bytes = if let Ok(bytes) = fs::read(file_path).await {
                bytes
            } else {
                warn!("Can't read the `{name}` plugin");
                continue;
            };
            let module = if let Ok(module) = Module::from_binary(store.engine(), &module_bytes) {
                module
            } else {
                warn!("Can't load the module from `{name}` plugin");
                continue;
            };

            match linker.instantiate_pre(&module) {
                Ok(pre) => match pre.instantiate_async(&mut store).await {
                    Ok(v) => {
                        instances.push((name.to_owned(), v));
                        info!(
                            "Initialized `{name}` in {:.2}s",
                            start.elapsed().as_secs_f32()
                        )
                    }
                    Err(err) => {
                        warn!("Can't instantiate_pre the module from `{name}` plugin: {err}")
                    }
                },
                Err(err) => warn!("Can't instantiated the module from `{name}` plugin: {err}"),
            }
        }

        Self {
            store,
            instances,
            process_media_info_rx,
        }
    }

    pub async fn process_media_info_json(&mut self, media: &mut MediaInfo) {
        if self.instances.is_empty() {
            return;
        }
        let json = serde_json::to_string(&media).unwrap();
        let mut store = self.store.as_context_mut();

        for (name, instance) in &self.instances {
            let process_media_info_json: wasmtime::TypedFunc<(u32, u32), ()> = if let Ok(func) =
                instance
                    .get_typed_func::<(u32, u32), ()>(&mut store, PROCESS_MEDIA_INFO_JSON_FUNCTION)
            {
                func
            } else {
                warn!("Can't get the `{PROCESS_MEDIA_INFO_JSON_FUNCTION}` from `{name}` plugin",);
                continue;
            };

            let memory = if let Some(memory) = instance.get_memory(&mut store, "memory") {
                memory
            } else {
                warn!("Can't get the memory from `{name}` plugin");
                continue;
            };

            let info_ptr = 0;
            let info_len = json.as_bytes().len();
            memory
                .write(&mut store, info_ptr, json.as_bytes())
                .expect("Write the media info to wasm memory failed!");

            info!(
                "[{name}] Processing `{}`, path: `{:?}`",
                media.title, media.path
            );
            let result = process_media_info_json
                .call_async(&mut store, (info_ptr as _, info_len as _))
                .await;

            match result {
                Ok(_) => {
                    if let Ok(data) = self.process_media_info_rx.try_recv() {
                        match serde_json::from_str(&data) {
                            Ok(modified_info) => *media = modified_info,
                            Err(err) => error!("Deserialize modified info failed: {err}"),
                        }
                    }
                }
                Err(err) => {
                    error!("Call `{PROCESS_MEDIA_INFO_JSON_FUNCTION}` error: {err}");
                }
            }
        }
    }
}

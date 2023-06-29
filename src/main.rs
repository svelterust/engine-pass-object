use anyhow::Result;
use serde_json::{json, Value as Json};
use wasmer::{Instance, Module, Store, Value};
use wasmer_wasi::WasiState;

struct Engine {
    store: wasmer::Store,
    instance: wasmer::Instance,
}

impl Engine {
    fn new(file: &str) -> Result<Self> {
        // Import the WASM file
        let mut store = Store::default();
        let module = Module::from_file(&store, file)?;

        // Create WASI environment
        let wasi_env = WasiState::new("engine").finalize(&mut store)?;
        let import_object = wasi_env.import_object(&mut store, &module)?;
        let instance = Instance::new(&mut store, &module, &import_object)?;

        // Pass memory reference to WASI
        let memory = instance.exports.get_memory("memory")?;
        wasi_env.data_mut(&mut store).set_memory(memory.clone());

        Ok(Self { store, instance })
    }

    fn run(&mut self, input: Json) -> Result<Box<[Value]>> {
        let function = self.instance.exports.get_function("main")?;
        let heap_start = 0x110000;

        // Serialize to bytes
        let serialized = rmp_serde::to_vec(&input)?;
        let input_len = (serialized.len() as u32).to_le_bytes();
        let input_bytes = [&input_len[..], &serialized].concat();

        // Grow memory for input
        let memory = self.instance.exports.get_memory("memory")?;
        let pages = (input_bytes.len() / wasmer::WASM_PAGE_SIZE) + 1;
        memory.grow(&mut self.store, pages as u32)?;

        // Write bytes into memory
        let view = memory.view(&self.store);
        view.write(heap_start as u64, &input_bytes)?;

        let result = function.call(&mut self.store, &[Value::I32(heap_start)])?;
        Ok(result)
    }
}

fn main() -> Result<()> {
    let file = std::env::args().skip(1).next().expect("No file was passed");
    let mut engine = Engine::new(&file)?;
    engine.run(json!({
        "name": "James",
        "age": 50,
        "money": 50.60,
    }))?;
    Ok(())
}

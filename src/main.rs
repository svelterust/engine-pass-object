use anyhow::Result;
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

    fn run(&mut self, params: &[Value]) -> Result<Box<[Value]>> {
        let function = self.instance.exports.get_function("main")?;
        let result = function.call(&mut self.store, params)?;
        Ok(result)
    }
}

fn main() -> Result<()> {
    let file = std::env::args().skip(1).next().expect("No file was passed");
    let mut engine = Engine::new(&file)?;
    engine.run(&[])?;
    Ok(())
}

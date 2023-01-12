use elements_ecs::EntityData;

use crate::shared::{
    bindings,
    guest_conversion::GuestConvert,
    interface::{guest, host},
    wasm::{GuestExports, WasmContext},
};

impl<Bindings: host::Host, Context: WasmContext<Bindings>> GuestExports<Bindings, Context>
    for crate::shared::interface::guest::Guest<Context>
{
    fn create(
        engine: &wasmtime::Engine,
        store: &mut wasmtime::Store<Context>,
        linker: &mut wasmtime::Linker<Context>,
        bytecode: &[u8],
    ) -> anyhow::Result<(Self, wasmtime::Instance)> {
        host::add_to_linker(linker, |cx| -> &mut Bindings {
            cx.bindings_implementation()
        })?;

        let module = wasmtime::Module::from_binary(engine, bytecode)?;
        guest::Guest::instantiate(store, &module, linker, |cx| cx.guest_data())
    }

    fn initialize(&self, store: &mut wasmtime::Store<Context>) -> anyhow::Result<()> {
        Ok(self.init(store)?)
    }

    fn run(
        &self,
        store: &mut wasmtime::Store<Context>,
        event_name: &str,
        components: &EntityData,
        time: f32,
        frametime: f32,
    ) -> anyhow::Result<()> {
        // remap the generated entitydata to components to send across
        let components = bindings::convert_entity_data_to_components(components);
        // TEMPORARY: convert the host rep components to owned guest rep components
        let components: Vec<_> = components
            .iter()
            .map(|(id, ct)| (*id, ct.guest_convert()))
            .collect();
        // then get the borrowing representation
        // these two steps should be unnecessary once we can update to the component version of wit-bindgen
        let components: Vec<_> = components
            .iter()
            .map(|(id, ct)| (*id, ct.as_guest()))
            .collect();

        Ok(self.exec(
            store,
            guest::RunContext { time, frametime },
            event_name,
            &components,
        )?)
    }
}

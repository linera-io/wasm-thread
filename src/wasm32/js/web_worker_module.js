// Wait for the main thread to send us the shared module/memory and work context.
// Once we've got it, initialize it all with the `wasm_bindgen` global we imported via
// `importScripts`.
self.onmessage = event => {
    let [ url, module, memory, work ] = event.data;

    (async () => {
        try {
            const wasm = await (await import(url)).initSync({ module, memory });
            console.log('Wasm initialized');
            // Enter rust code by calling entry point defined in `lib.rs`.
            // This executes closure defined by work context.
            await wasm.wasm_thread_entry_point(work);

            console.log('Work completed successfully');
        } catch (err) {
            console.error(err);

            // Propagate to main `onerror`:
            setTimeout(() => {
                throw err;
            });
            // Rethrow to keep promise rejected and prevent execution of further commands:
            throw err;
        }

        console.log('Closing worker');

        // Once done, terminate web worker
        close();
    })();
};

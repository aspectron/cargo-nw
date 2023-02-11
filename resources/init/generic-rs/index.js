(async()=>{
    window.$$SNAKE = await import('/app/wasm/___NAME___.js');
    // window.$$SNAKE = $$NAME;
    const wasm = await window.$$SNAKE.default('/app/wasm/___NAME____bg.wasm');
    //console.log("wasm", wasm, workflow)
    //$$SNAKE.init_console_panic_hook();
    //$$SNAKE.show_panic_hook_logs();
    window.$$SNAKE.initialize();
})();

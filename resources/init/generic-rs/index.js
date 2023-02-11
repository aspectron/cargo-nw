(async()=>{
    window.$$SNAKE = await import('/root/wasm/$NAME.js');
    // window.$$SNAKE = $$NAME;
    const wasm = await window.$$SNAKE.default('/root/wasm/$NAME_bg.wasm');
    //console.log("wasm", wasm, workflow)
    //$$SNAKE.init_console_panic_hook();
    //$$SNAKE.show_panic_hook_logs();
    window.$$SNAKE.initialize();
})();

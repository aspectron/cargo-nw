(async()=>{
    window.$$SNAKE = await import('/app/wasm/___NAME___.js');
    const wasm = await window.$$SNAKE.default('/app/wasm/___NAME____bg.wasm');
    $$SNAKE.init_console_panic_hook();
    window.$$SNAKE.initialize();
})();

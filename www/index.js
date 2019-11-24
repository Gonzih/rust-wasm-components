import * as wasm from "rust-wasm-components";

let framework = wasm.run();
let cb = function() {
    framework.tick();
    window.requestAnimationFrame(cb);
};

// cb();

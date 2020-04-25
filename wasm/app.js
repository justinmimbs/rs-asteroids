
let wasm;

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachegetUint8Memory0 = null;
function getUint8Memory0() {
    if (cachegetUint8Memory0 === null || cachegetUint8Memory0.buffer !== wasm.memory.buffer) {
        cachegetUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}
/**
*/
export class App {

    static __wrap(ptr) {
        const obj = Object.create(App.prototype);
        obj.ptr = ptr;

        return obj;
    }

    free() {
        const ptr = this.ptr;
        this.ptr = 0;

        wasm.__wbg_app_free(ptr);
    }
    /**
    * @returns {App}
    */
    static new() {
        var ret = wasm.app_new();
        return App.__wrap(ret);
    }
    /**
    * @param {number} dt
    * @param {number} input
    */
    step(dt, input) {
        wasm.app_step(this.ptr, dt, input);
    }
    /**
    * @returns {PathList}
    */
    render() {
        var ret = wasm.app_render(this.ptr);
        return PathList.__wrap(ret);
    }
}
/**
*/
export class PathList {

    static __wrap(ptr) {
        const obj = Object.create(PathList.prototype);
        obj.ptr = ptr;

        return obj;
    }

    free() {
        const ptr = this.ptr;
        this.ptr = 0;

        wasm.__wbg_pathlist_free(ptr);
    }
    /**
    * @returns {number}
    */
    length() {
        var ret = wasm.pathlist_length(this.ptr);
        return ret >>> 0;
    }
    /**
    * @returns {number}
    */
    paths() {
        var ret = wasm.pathlist_paths(this.ptr);
        return ret;
    }
    /**
    * @returns {number}
    */
    alphas() {
        var ret = wasm.pathlist_alphas(this.ptr);
        return ret;
    }
    /**
    * @returns {number}
    */
    ends() {
        var ret = wasm.pathlist_ends(this.ptr);
        return ret;
    }
    /**
    * @returns {number}
    */
    points_length() {
        var ret = wasm.pathlist_points_length(this.ptr);
        return ret >>> 0;
    }
    /**
    * @returns {number}
    */
    points() {
        var ret = wasm.pathlist_points(this.ptr);
        return ret;
    }
}

async function load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {

        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {

        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

async function init(input) {
    if (typeof input === 'undefined') {
        input = import.meta.url.replace(/\.js$/, '_bg.wasm');
    }
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };

    if (typeof input === 'string' || (typeof Request === 'function' && input instanceof Request) || (typeof URL === 'function' && input instanceof URL)) {
        input = fetch(input);
    }

    const { instance, module } = await load(await input, imports);

    wasm = instance.exports;
    init.__wbindgen_wasm_module = module;

    return wasm;
}

export default init;


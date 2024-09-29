/* tslint:disable */
/* eslint-disable */
/**
* @param {number} lng
* @param {number} zoom
* @returns {bigint}
*/
export function lng_to_tile_x(lng: number, zoom: number): bigint;
/**
* @param {number} lat
* @param {number} zoom
* @returns {bigint}
*/
export function lat_to_tile_y(lat: number, zoom: number): bigint;
/**
*/
export class FogMap {
  free(): void;
/**
*/
  constructor();
/**
* @param {string} file_name
* @param {Uint8Array} data
*/
  add_fow_file(file_name: string, data: Uint8Array): void;
}
/**
*/
export class FogRenderer {
  free(): void;
/**
*/
  constructor();
/**
* @param {FogMap} fogmap
* @param {bigint} view_x
* @param {bigint} view_y
* @param {number} zoom
* @returns {Uint8Array}
*/
  render_image(fogmap: FogMap, view_x: bigint, view_y: bigint, zoom: number): Uint8Array;
/**
* @param {FogMap} fogmap
* @param {bigint} view_x
* @param {bigint} view_y
* @param {number} zoom
* @returns {Uint8Array}
*/
  render_image_raw(fogmap: FogMap, view_x: bigint, view_y: bigint, zoom: number): Uint8Array;
/**
* @param {FogMap} fogmap
* @param {bigint} view_x
* @param {bigint} view_y
* @param {number} zoom
* @returns {Uint8Array}
*/
  render_and_blur_image(fogmap: FogMap, view_x: bigint, view_y: bigint, zoom: number): Uint8Array;
}
/**
*/
export class GpuFogRenderer {
  free(): void;
/**
* @param {number} width
* @param {number} height
* @returns {Promise<any>}
*/
  static create(width: number, height: number): Promise<any>;
/**
* @param {FogMap} fogmap
* @param {bigint} view_x
* @param {bigint} view_y
* @param {number} zoom
* @param {Function} callback
*/
  render_image(fogmap: FogMap, view_x: bigint, view_y: bigint, zoom: number, callback: Function): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_fogmap_free: (a: number, b: number) => void;
  readonly fogmap_new: () => number;
  readonly fogmap_add_fow_file: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly __wbg_fogrenderer_free: (a: number, b: number) => void;
  readonly fogrenderer_new: () => number;
  readonly fogrenderer_render_image: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly fogrenderer_render_image_raw: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly fogrenderer_render_and_blur_image: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly __wbg_gpufogrenderer_free: (a: number, b: number) => void;
  readonly gpufogrenderer_create: (a: number, b: number) => number;
  readonly gpufogrenderer_render_image: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly lng_to_tile_x: (a: number, b: number) => number;
  readonly lat_to_tile_y: (a: number, b: number) => number;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h20296e3f757f2527: (a: number, b: number, c: number) => void;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__ha5394284015dee35: (a: number, b: number, c: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly wasm_bindgen__convert__closures__invoke2_mut__h4c5fdaf53d786397: (a: number, b: number, c: number, d: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;

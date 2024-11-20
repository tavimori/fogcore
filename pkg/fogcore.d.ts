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
 * Sample position for subsampled chroma
 */
export enum ChromaSamplePosition {
  /**
   * The source video transfer function must be signaled
   * outside the AV1 bitstream.
   */
  Unknown = 0,
  /**
   * Horizontally co-located with (0, 0) luma sample, vertically positioned
   * in the middle between two luma samples.
   */
  Vertical = 1,
  /**
   * Co-located with (0, 0) luma sample.
   */
  Colocated = 2,
}
/**
 * Chroma subsampling format
 */
export enum ChromaSampling {
  /**
   * Both vertically and horizontally subsampled.
   */
  Cs420 = 0,
  /**
   * Horizontally subsampled.
   */
  Cs422 = 1,
  /**
   * Not subsampled.
   */
  Cs444 = 2,
  /**
   * Monochrome.
   */
  Cs400 = 3,
}
/**
 * Allowed pixel value range
 *
 * C.f. `VideoFullRangeFlag` variable specified in ISO/IEC 23091-4/ITU-T H.273
 */
export enum PixelRange {
  /**
   * Studio swing representation
   */
  Limited = 0,
  /**
   * Full swing representation
   */
  Full = 1,
}
export enum Tune {
  Psnr = 0,
  Psychovisual = 1,
}
export class FogMap {
  free(): void;
  /**
   * @returns {Promise<any>}
   */
  static new(): Promise<any>;
  /**
   * @returns {Promise<any>}
   */
  static new_no_renderer(): Promise<any>;
  /**
   * @param {string} file_name
   * @param {Uint8Array} data
   */
  add_fow_file(file_name: string, data: Uint8Array): void;
  /**
   * @param {Uint8Array} data
   */
  add_fow_zip(data: Uint8Array): void;
  /**
   * @param {number} sw_x
   * @param {number} sw_y
   * @param {number} ne_x
   * @param {number} ne_y
   * @returns {Float32Array}
   */
  get_bounding_mercator_pixels(sw_x: number, sw_y: number, ne_x: number, ne_y: number): Float32Array;
  /**
   * @param {bigint} view_x
   * @param {bigint} view_y
   * @param {number} zoom
   * @returns {Promise<Uint8Array>}
   */
  render_image(view_x: bigint, view_y: bigint, zoom: number): Promise<Uint8Array>;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly lng_to_tile_x: (a: number, b: number) => number;
  readonly lat_to_tile_y: (a: number, b: number) => number;
  readonly __wbg_fogmap_free: (a: number, b: number) => void;
  readonly fogmap_new: () => number;
  readonly fogmap_new_no_renderer: () => number;
  readonly fogmap_add_fow_file: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly fogmap_add_fow_zip: (a: number, b: number, c: number) => void;
  readonly fogmap_get_bounding_mercator_pixels: (a: number, b: number, c: number, d: number, e: number) => Array;
  readonly fogmap_render_image: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_export_3: WebAssembly.Table;
  readonly __wbindgen_export_4: WebAssembly.Table;
  readonly closure304_externref_shim: (a: number, b: number, c: number) => void;
  readonly closure315_externref_shim: (a: number, b: number, c: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly closure330_externref_shim: (a: number, b: number, c: number, d: number) => void;
  readonly __wbindgen_start: () => void;
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

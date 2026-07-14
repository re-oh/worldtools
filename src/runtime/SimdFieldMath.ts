import type { BulkFieldMath } from "../lib/field/BulkFieldMath";
import { WORLD_FORMAT_ABI, WORLD_VERSION } from "../world/recipe";

interface BomboWasmExports extends WebAssembly.Exports {
  memory: WebAssembly.Memory;
  bombo_schema_version(): number;
  bombo_alloc_f32(length: number): number;
  bombo_free_f32(pointer: number, capacity: number): void;
  bombo_affine_clamp_f32(pointer: number, length: number, scale: number, bias: number, minimum: number, maximum: number): number;
  bombo_max_f32(target: number, source: number, length: number): number;
}

class SimdFieldMath implements BulkFieldMath {
  readonly backend = "Rust/WASM SIMD128";

  constructor(private readonly wasm: BomboWasmExports) {
    if (wasm.bombo_schema_version() !== WORLD_FORMAT_ABI) {
      throw new Error(`The SIMD field kernel ABI does not match world format ${WORLD_VERSION}.`);
    }
  }

  affineClamp(values: Float32Array, scale: number, bias: number, minimum: number, maximum: number): void {
    const pointer = this.wasm.bombo_alloc_f32(values.length);
    try {
      this.view(pointer, values.length).set(values);
      const status = this.wasm.bombo_affine_clamp_f32(pointer, values.length, scale, bias, minimum, maximum);
      if (status !== 0) throw new Error(`SIMD affine kernel rejected the field (${status}).`);
      values.set(this.view(pointer, values.length));
    } finally {
      this.wasm.bombo_free_f32(pointer, values.length);
    }
  }

  maxInto(target: Float32Array, source: Float32Array): void {
    this.maxManyInto(target, [source]);
  }

  maxManyInto(target: Float32Array, sources: readonly Float32Array[]): void {
    for (const source of sources) {
      if (target.length !== source.length) throw new Error("Bulk maximum fields must have equal length.");
    }
    if (sources.length === 0 || target.length === 0) return;
    const targetPointer = this.wasm.bombo_alloc_f32(target.length);
    const sourcePointer = this.wasm.bombo_alloc_f32(target.length);
    try {
      this.view(targetPointer, target.length).set(target);
      for (const source of sources) {
        this.view(sourcePointer, source.length).set(source);
        const status = this.wasm.bombo_max_f32(targetPointer, sourcePointer, target.length);
        if (status !== 0) throw new Error(`SIMD maximum kernel rejected the fields (${status}).`);
      }
      target.set(this.view(targetPointer, target.length));
    } finally {
      this.wasm.bombo_free_f32(sourcePointer, target.length);
      this.wasm.bombo_free_f32(targetPointer, target.length);
    }
  }

  private view(pointer: number, length: number): Float32Array {
    return new Float32Array(this.wasm.memory.buffer, pointer, length);
  }
}

let loading: Promise<BulkFieldMath> | null = null;

export function loadSimdFieldMath(): Promise<BulkFieldMath> {
  loading ??= instantiate().then((exports) => new SimdFieldMath(exports));
  return loading;
}

async function instantiate(): Promise<BomboWasmExports> {
  const response = await fetch("/wasm/bombo_core.wasm");
  if (!response.ok) throw new Error(`SIMD field kernel request failed with HTTP ${response.status}.`);
  const result = await WebAssembly.instantiate(await response.arrayBuffer(), {});
  return result.instance.exports as BomboWasmExports;
}

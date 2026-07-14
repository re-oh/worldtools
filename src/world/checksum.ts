import type { FieldMap } from "./model";
import { FIELD_IDS } from "./model";

export function checksumFields(fields: FieldMap, seed = 2166136261): string {
  let a = seed >>> 0;
  let b = (seed ^ 0x9e3779b9) >>> 0;
  for (const id of FIELD_IDS) {
    const words = new Uint32Array(fields[id].buffer, fields[id].byteOffset, fields[id].byteLength / 4);
    for (let index = 0; index < words.length; index += 1) {
      a = Math.imul(a ^ words[index], 16777619) >>> 0;
      b = Math.imul(b ^ ((words[index] << 16) | (words[index] >>> 16)), 2246822519) >>> 0;
    }
  }
  return `${a.toString(16).padStart(8, "0")}${b.toString(16).padStart(8, "0")}`;
}

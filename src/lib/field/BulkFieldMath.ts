export interface BulkFieldMath {
  readonly backend: string;
  affineClamp(values: Float32Array, scale: number, bias: number, minimum: number, maximum: number): void;
  maxInto(target: Float32Array, source: Float32Array): void;
  maxManyInto(target: Float32Array, sources: readonly Float32Array[]): void;
}

export const SCALAR_BULK_FIELD_MATH: BulkFieldMath = {
  backend: "TypeScript scalar",
  affineClamp(values, scale, bias, minimum, maximum) {
    for (let index = 0; index < values.length; index += 1) values[index] = Math.max(minimum, Math.min(maximum, values[index] * scale + bias));
  },
  maxInto(target, source) {
    if (target.length !== source.length) throw new Error("Bulk maximum fields must have equal length.");
    for (let index = 0; index < target.length; index += 1) target[index] = Math.max(target[index], source[index]);
  },
  maxManyInto(target, sources) {
    for (const source of sources) {
      if (target.length !== source.length) throw new Error("Bulk maximum fields must have equal length.");
    }
    for (let index = 0; index < target.length; index += 1) {
      let maximum = target[index];
      for (const source of sources) maximum = Math.max(maximum, source[index]);
      target[index] = maximum;
    }
  }
};

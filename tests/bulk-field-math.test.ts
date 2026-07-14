import { describe, expect, it } from "vitest";
import { SCALAR_BULK_FIELD_MATH } from "../src/lib/field/BulkFieldMath";

describe("bulk field math", () => {
  it("reduces multiple source fields into one maximum field", () => {
    const target = new Float32Array([1, 5, -2, 4]);
    const sources = [
      new Float32Array([3, 2, -4, 8]),
      new Float32Array([2, 9, 6, 1])
    ];

    SCALAR_BULK_FIELD_MATH.maxManyInto(target, sources);

    expect(Array.from(target)).toEqual([3, 9, 6, 8]);
  });

  it("rejects mismatched source lengths before mutating the target", () => {
    const target = new Float32Array([1, 2, 3]);

    expect(() => SCALAR_BULK_FIELD_MATH.maxManyInto(target, [new Float32Array([5, 6])])).toThrow(/equal length/);
    expect(Array.from(target)).toEqual([1, 2, 3]);
  });
});

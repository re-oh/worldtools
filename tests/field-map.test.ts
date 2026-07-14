import { describe, expect, it } from "vitest";
import { createEmptyFieldMap } from "../src/world/model";

describe("packed field map", () => {
  it("materializes independent zero fields and accepts validated stage output", () => {
    const fields = createEmptyFieldMap(4);
    fields.elevation[0] = 12;
    expect(Array.from(fields.flow)).toEqual([0, 0, 0, 0]);

    const replacement = new Float32Array([1, 2, 3, 4]);
    fields.flow = replacement;
    expect(fields.flow).toBe(replacement);
  });

  it("rejects stage output with the wrong cell count", () => {
    const fields = createEmptyFieldMap(4);
    expect(() => { fields.flow = new Float32Array(3); }).toThrow(/4 cells/);
  });
});

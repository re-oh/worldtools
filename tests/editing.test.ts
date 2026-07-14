import { describe, expect, it } from "vitest";
import { CommandHistory } from "../src/lib/history/CommandHistory";
import { createConstraintSet, constraintCount } from "../src/world/constraints";
import { DEFAULT_RECIPE } from "../src/world/recipe";
import { ConstraintPainter } from "../src/workbench/editing/ConstraintPainter";

describe("manual constraint editing", () => {
  it("commits geodesic brush strokes as undoable commands", () => {
    const recipe = { ...DEFAULT_RECIPE, width: 128, height: 64 };
    const constraints = createConstraintSet(recipe.width * recipe.height);
    const history = new CommandHistory();
    let changes = 0;
    const painter = new ConstraintPainter(history, () => { changes += 1; });
    painter.setWorld(recipe, constraints);
    painter.setSettings({ constraint: "elevation", radiusKm: 500, strength: 1, sign: 1 });
    painter.begin(32 * recipe.width + 64);
    expect(painter.end()).toBe(true);
    const paintedCount = constraintCount(constraints);
    expect(paintedCount).toBeGreaterThan(5);
    expect(history.canUndo).toBe(true);
    history.undo();
    expect(constraintCount(constraints)).toBe(0);
    history.redo();
    expect(constraintCount(constraints)).toBe(paintedCount);
    expect(changes).toBeGreaterThanOrEqual(3);
  });
});

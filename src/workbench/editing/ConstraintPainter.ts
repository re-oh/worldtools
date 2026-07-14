import { createGridGeometry, greatCircleDistanceKm, type GridGeometry } from "../../lib/field/grid";
import { clamp, lerp } from "../../lib/math/scalar";
import { CommandHistory } from "../../lib/history/CommandHistory";
import { CONSTRAINT_DEFINITIONS, type ConstraintId, type ConstraintSet } from "../../world/constraints";
import type { WorldRecipe } from "../../world/recipe";

export type PaintSign = -1 | 0 | 1;

export interface PaintSettings {
  constraint: ConstraintId;
  radiusKm: number;
  strength: number;
  sign: PaintSign;
}

export class ConstraintPainter {
  private recipe: WorldRecipe | null = null;
  private constraints: ConstraintSet | null = null;
  private grid: GridGeometry | null = null;
  private settings: PaintSettings = { constraint: "elevation", radiusKm: 350, strength: 1, sign: 1 };
  private before = new Map<number, number>();
  private lastCell = -1;

  constructor(private readonly history: CommandHistory, private readonly onChange: (constraint: ConstraintId) => void) {}

  setWorld(recipe: WorldRecipe, constraints: ConstraintSet): void {
    this.recipe = recipe;
    this.constraints = constraints;
    this.grid = createGridGeometry(recipe.width, recipe.height, recipe.radiusKm);
  }

  setSettings(settings: Partial<PaintSettings>): void {
    this.settings = { ...this.settings, ...settings };
  }

  begin(cell: number): void {
    this.before.clear();
    this.lastCell = -1;
    this.paint(cell);
  }

  paint(cell: number): void {
    if (!this.recipe || !this.constraints || !this.grid || cell === this.lastCell) return;
    this.lastCell = cell;
    const definition = CONSTRAINT_DEFINITIONS[this.settings.constraint];
    const values = this.constraints[this.settings.constraint];
    const centerRow = Math.floor(cell / this.grid.width);
    const centerColumn = cell % this.grid.width;
    const nominalKm = Math.sqrt(this.grid.cellAreaKm2);
    const rowSpan = Math.max(1, Math.ceil(this.settings.radiusKm / nominalKm * 1.8));
    const cosine = Math.max(0.12, this.grid.cosLatitude[centerRow]);
    const columnSpan = Math.min(this.grid.width / 2, Math.max(1, Math.ceil(this.settings.radiusKm / (nominalKm * cosine) * 1.5)));
    for (let row = Math.max(0, centerRow - rowSpan); row <= Math.min(this.grid.height - 1, centerRow + rowSpan); row += 1) {
      for (let offset = -columnSpan; offset <= columnSpan; offset += 1) {
        const column: number = ((centerColumn + offset) % this.grid.width + this.grid.width) % this.grid.width;
        const target = row * this.grid.width + column;
        const distance = greatCircleDistanceKm(cell, target, this.grid);
        if (distance > this.settings.radiusKm) continue;
        const normalized = distance / Math.max(1, this.settings.radiusKm);
        const falloff = Math.pow(1 - normalized * normalized, 2);
        if (!this.before.has(target)) this.before.set(target, values[target]);
        if (this.settings.sign === 0) {
          values[target] = lerp(values[target], 0, Math.min(1, this.settings.strength * falloff * 0.34));
        } else {
          const delta = definition.defaultBrushStrength * this.settings.strength * this.settings.sign * falloff * 0.32;
          values[target] = clamp(values[target] + delta, definition.minimum, definition.maximum);
        }
      }
    }
  }

  end(): boolean {
    if (!this.constraints || this.before.size === 0) return false;
    const values = this.constraints[this.settings.constraint];
    const indices = Int32Array.from(this.before.keys());
    const before = Float32Array.from(indices, (index) => this.before.get(index)!);
    const after = Float32Array.from(indices, (index) => values[index]);
    const constraint = this.settings.constraint;
    const apply = (snapshot: Float32Array) => {
      const target = this.constraints?.[constraint];
      if (!target) return;
      for (let offset = 0; offset < indices.length; offset += 1) target[indices[offset]] = snapshot[offset];
      this.onChange(constraint);
    };
    this.history.commit({ label: `Paint ${CONSTRAINT_DEFINITIONS[constraint].label}`, undo: () => apply(before), redo: () => apply(after) });
    this.before.clear();
    this.onChange(constraint);
    return true;
  }
}

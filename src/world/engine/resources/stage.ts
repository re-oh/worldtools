import type { WorldEngineStage } from "../stage";
import { evaluateResourceModels } from "./evaluator";
import { INDUSTRIAL_RESOURCE_MODELS } from "./models/industrial";
import { BEDROCK_MINERAL_MODELS } from "./models/minerals";
import { ORGANIC_RESOURCE_MODELS } from "./models/organic";
import { createPlacerModels, transportDenseMinerals } from "./transport";

export const resourcesStage: WorldEngineStage = {
  id: "resources",
  label: "Natural resource systems",
  detail: "Evaluating geological process chains, preservation, maturity, and secondary concentration",
  run(context, report) {
    context.deposits.push(...evaluateResourceModels(context, BEDROCK_MINERAL_MODELS));
    report(0.38, "Primary metallic mineral systems evaluated");
    context.deposits.push(...evaluateResourceModels(context, INDUSTRIAL_RESOURCE_MODELS));
    report(0.62, "Clay, evaporite, nitrate, and primary gemstone systems evaluated");
    context.deposits.push(...evaluateResourceModels(context, ORGANIC_RESOURCE_MODELS));
    report(0.84, "Peat, coal rank, petroleum, and natural-gas systems evaluated");

    const goldSource = new Float32Array(context.grid.cellCount);
    const gemSource = context.fields.gemstone.slice();
    for (let cell = 0; cell < context.grid.cellCount; cell += 1) goldSource[cell] = Math.max(context.fields.orogenicGold[cell], context.fields.gold[cell]);
    const goldTransport = transportDenseMinerals(context, goldSource);
    const gemTransport = transportDenseMinerals(context, gemSource);
    context.deposits.push(...evaluateResourceModels(context, createPlacerModels(goldTransport, gemTransport)));

    const fields = context.fields;
    context.bulkMath.maxManyInto(fields.mineralPotential, [fields.bandedIron, fields.bauxite, fields.copperSulfide, fields.magmaticSulfide, fields.carbonatite, fields.orogenicGold, fields.copper, fields.gold, fields.iron, fields.rareEarth, fields.nickel, fields.placer]);
    fields.resourcePotential.set(fields.mineralPotential);
    context.bulkMath.maxManyInto(fields.resourcePotential, [fields.clayDeposit, fields.peat, fields.coal, fields.hydrocarbon, fields.evaporite, fields.nitrate, fields.gemstone]);
    context.deposits.sort((a, b) => b.potential - a.potential || a.typeId.localeCompare(b.typeId) || a.cell - b.cell);
    report(1, "Secondary placers and combined resource potential complete");
  }
};

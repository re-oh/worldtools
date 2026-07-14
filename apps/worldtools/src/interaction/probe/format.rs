use worldtools_simulation::WorldSample;
use worldtools_ui::{LayerProbe, WorldLayer};

use super::{percent, wind_reading};

pub(super) fn populate(probe: &mut LayerProbe, layer: WorldLayer, sample: WorldSample) {
    match layer {
        WorldLayer::Elevation => {}
        WorldLayer::Tectonics => tectonics(probe, sample),
        WorldLayer::Hydrology => hydrology(probe, sample),
        WorldLayer::Climate => climate(probe, sample),
        WorldLayer::Soil => soil(probe, sample),
        WorldLayer::Vegetation => vegetation(probe, sample),
        WorldLayer::Geology => geology(probe, sample),
        WorldLayer::Resources => resources(probe, sample),
    }
}

fn tectonics(probe: &mut LayerProbe, sample: WorldSample) {
    let data = sample.tectonics;
    probe.push_reading("Plate", format!("#{}", data.plate_id));
    probe.push_reading("Paleo plate", format!("#{}", data.paleo_plate_id));
    probe.push_reading("Terrane", format!("#{}", data.terrane_id));
    probe.push_reading("Crust", data.crust.label());
    probe.push_reading("Boundary type", data.boundary_kind.label());
    probe.push_reading("Crust age", format!("{:.0} Myr", data.crust_age_myr));
    probe.push_reading(
        "Crust thickness",
        format!("{:.1} km", data.crust_thickness_km),
    );
    probe.push_reading("Boundary", percent(data.boundary));
    probe.push_reading("Convergence", percent(data.convergence));
    probe.push_reading("Divergence", percent(data.divergence));
    probe.push_reading("Transform shear", percent(data.shear));
    probe.push_reading("Inherited suture", percent(data.suture));
    probe.push_reading("Metamorphic grade", percent(data.metamorphic_grade));
    probe.push_reading("Uplift", format!("{:+.0} m", data.uplift_m));
    probe.push_reading("Volcanism", percent(data.volcanism));
}

fn hydrology(probe: &mut LayerProbe, sample: WorldSample) {
    let data = sample.hydrology;
    probe.push_reading("River", percent(data.river_strength));
    probe.push_reading("Runoff", percent(data.runoff));
    probe.push_reading("Wetness", percent(data.wetness));
    probe.push_reading("Lake", percent(data.lake));
    probe.push_reading("Erosion", format!("{:.1} m", data.erosion_m));
    probe.push_reading("Sediment", format!("{:.1} m", data.sediment_m));
    probe.push_reading("Glacial maximum", percent(data.maximum_ice_fraction));
    probe.push_reading("Ice flux", percent(data.ice_flux));
    probe.push_reading(
        "Glacial erosion",
        format!("{:.1} m", data.glacial_erosion_m),
    );
    probe.push_reading("Till", format!("{:.1} m", data.till_m));
    probe.push_reading("Outwash", format!("{:.1} m", data.outwash_m));
    probe.push_reading(
        "Isostatic rebound",
        format!("{:.1} m", data.isostatic_rebound_m),
    );
}

fn climate(probe: &mut LayerProbe, sample: WorldSample) {
    let data = sample.climate;
    probe.push_reading("Climate", data.zone.label());
    probe.push_reading("Temperature", format!("{:+.1} C", data.temperature_c));
    probe.push_reading(
        "Precipitation",
        format!("{:.0} mm/yr", data.precipitation_mm),
    );
    probe.push_reading("Seasonality", percent(data.seasonality));
    probe.push_reading("Wind", wind_reading(data.wind_east, data.wind_north));
    probe.push_reading("Aridity", percent(data.aridity));
}

fn soil(probe: &mut LayerProbe, sample: WorldSample) {
    let data = sample.soil;
    probe.push_reading("Soil", data.kind.label());
    probe.push_reading("Depth", format!("{:.2} m", data.depth_m));
    probe.push_reading("Fertility", percent(data.fertility));
    probe.push_reading("Clay", percent(data.clay_fraction));
    probe.push_reading("Organic", percent(data.organic_fraction));
    probe.push_reading("Drainage", percent(data.drainage));
}

fn vegetation(probe: &mut LayerProbe, sample: WorldSample) {
    let data = sample.vegetation;
    probe.push_reading("Biome", data.biome.label());
    probe.push_reading("Canopy", percent(data.canopy_fraction));
    probe.push_reading("Grass", percent(data.grass_fraction));
    probe.push_reading("Biomass", percent(data.biomass));
    probe.push_reading("Fire frequency", percent(data.fire_frequency));
}

fn geology(probe: &mut LayerProbe, sample: WorldSample) {
    let data = sample.geology;
    probe.push_reading("Bedrock", data.lithology.label());
    probe.push_reading("Rock age", format!("{:.0} Myr", data.rock_age_myr));
    probe.push_reading("Sediment", format!("{:.1} m", data.sediment_m));
    probe.push_reading("Volcanic ash", format!("{:.2} m", data.volcanic_ash_m));
    probe.push_reading("Weathering", percent(data.weathering));
}

fn resources(probe: &mut LayerProbe, sample: WorldSample) {
    let data = sample.resources;
    probe.push_reading("Deposit", data.dominant.label());
    probe.push_reading("Richness", percent(data.richness));
    probe.push_reading("Depth", format!("{:.0} m", data.depth_m));
    probe.push_reading("Confidence", percent(data.confidence));
    probe.push_reading("Metallic", percent(data.metallic));
    probe.push_reading("Energy", percent(data.energy));
    probe.push_reading("Industrial", percent(data.industrial));
}

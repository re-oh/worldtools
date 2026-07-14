#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct TerrainTileMaterialParams {
    sample_rect: vec4<f32>,
    metrics: vec4<f32>,
    debug: vec4<f32>,
    display: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> params: TerrainTileMaterialParams;
@group(#{MATERIAL_BIND_GROUP}) @binding(1)
var elevation_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2)
var blue_noise_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3)
var layer_texture: texture_2d<f32>;

fn load_height(pixel: vec2<i32>) -> f32 {
    let size = vec2<i32>(textureDimensions(elevation_texture, 0));
    return textureLoad(elevation_texture, clamp(pixel, vec2<i32>(0), size - 1), 0).r;
}

fn cubic(a: f32, b: f32, c: f32, d: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    return 0.5 * ((2.0 * b) + (-a + c) * t +
        (2.0 * a - 5.0 * b + 4.0 * c - d) * t2 +
        (-a + 3.0 * b - 3.0 * c + d) * t3);
}

fn cubic_derivative(a: f32, b: f32, c: f32, d: f32, t: f32) -> f32 {
    let t2 = t * t;
    return 0.5 * ((-a + c) +
        2.0 * (2.0 * a - 5.0 * b + 4.0 * c - d) * t +
        3.0 * (-a + 3.0 * b - 3.0 * c + d) * t2);
}

// Returns height and its X/Y derivative from a single 4x4 footprint.
fn sample_surface(uv: vec2<f32>) -> vec3<f32> {
    let point = params.sample_rect.xy + uv * params.sample_rect.zw;
    let base = vec2<i32>(floor(point));
    let fraction = fract(point);
    var rows: array<f32, 4>;
    var dx_rows: array<f32, 4>;

    for (var row = 0i; row < 4i; row += 1i) {
        let y = base.y + row - 1i;
        let a = load_height(vec2<i32>(base.x - 1i, y));
        let b = load_height(vec2<i32>(base.x, y));
        let c = load_height(vec2<i32>(base.x + 1i, y));
        let d = load_height(vec2<i32>(base.x + 2i, y));
        rows[u32(row)] = cubic(a, b, c, d, fraction.x);
        dx_rows[u32(row)] = cubic_derivative(a, b, c, d, fraction.x);
    }

    let height = cubic(rows[0], rows[1], rows[2], rows[3], fraction.y);
    let derivative_x = cubic(dx_rows[0], dx_rows[1], dx_rows[2], dx_rows[3], fraction.y);
    let derivative_y = cubic_derivative(rows[0], rows[1], rows[2], rows[3], fraction.y);
    return vec3<f32>(height, derivative_x, derivative_y);
}

fn load_layer(pixel: vec2<i32>) -> vec4<f32> {
    let size = vec2<i32>(textureDimensions(layer_texture, 0));
    return textureLoad(layer_texture, clamp(pixel, vec2<i32>(0), size - 1), 0);
}

fn cubic_vec4(
    a: vec4<f32>,
    b: vec4<f32>,
    c: vec4<f32>,
    d: vec4<f32>,
    t: f32,
) -> vec4<f32> {
    let t2 = t * t;
    let t3 = t2 * t;
    return 0.5 * ((2.0 * b) + (-a + c) * t +
        (2.0 * a - 5.0 * b + 4.0 * c - d) * t2 +
        (-a + 3.0 * b - 3.0 * c + d) * t3);
}

// Continuous layer values use the same bicubic footprint as elevation.
fn sample_layer(uv: vec2<f32>) -> vec4<f32> {
    let point = params.sample_rect.xy + uv * params.sample_rect.zw;
    let base = vec2<i32>(floor(point));
    let fraction = fract(point);
    var rows: array<vec4<f32>, 4>;

    for (var row = 0i; row < 4i; row += 1i) {
        let y = base.y + row - 1i;
        rows[u32(row)] = cubic_vec4(
            load_layer(vec2<i32>(base.x - 1i, y)),
            load_layer(vec2<i32>(base.x, y)),
            load_layer(vec2<i32>(base.x + 1i, y)),
            load_layer(vec2<i32>(base.x + 2i, y)),
            fraction.x,
        );
    }

    return cubic_vec4(rows[0], rows[1], rows[2], rows[3], fraction.y);
}

// Category IDs must remain exact at boundaries rather than becoming fractional.
fn sample_layer_kind(uv: vec2<f32>) -> f32 {
    let point = params.sample_rect.xy + uv * params.sample_rect.zw;
    return load_layer(vec2<i32>(floor(point + vec2<f32>(0.5)))).x;
}

fn unit_or_scaled(value: f32, scale: f32) -> f32 {
    let positive = max(value, 0.0);
    if (positive <= 1.0) {
        return positive;
    }
    return 1.0 - exp(-positive / scale);
}

fn plate_palette_color(kind_value: f32, palette: array<vec3<f32>, 12>) -> vec3<f32> {
    let kind = u32(max(round(kind_value), 0.0));
    return palette[kind % 12u];
}

fn plate_color(plate_id: f32) -> vec3<f32> {
    return plate_palette_color(plate_id, array<vec3<f32>, 12>(
        vec3<f32>(0.16, 0.34, 0.45),
        vec3<f32>(0.46, 0.25, 0.18),
        vec3<f32>(0.20, 0.40, 0.25),
        vec3<f32>(0.49, 0.38, 0.15),
        vec3<f32>(0.34, 0.23, 0.48),
        vec3<f32>(0.14, 0.42, 0.42),
        vec3<f32>(0.49, 0.20, 0.29),
        vec3<f32>(0.34, 0.43, 0.16),
        vec3<f32>(0.23, 0.29, 0.52),
        vec3<f32>(0.53, 0.30, 0.12),
        vec3<f32>(0.16, 0.36, 0.31),
        vec3<f32>(0.43, 0.19, 0.43),
    ));
}

fn soil_color(kind: f32) -> vec3<f32> {
    let index = min(u32(max(round(kind), 0.0)), 10u);
    let palette = array<vec3<f32>, 11>(
        vec3<f32>(0.12, 0.28, 0.33), // Ocean / marine sediment
        vec3<f32>(0.33, 0.31, 0.28), // Bare rock
        vec3<f32>(0.54, 0.61, 0.62), // Cryosol
        vec3<f32>(0.77, 0.56, 0.24), // Desert soil
        vec3<f32>(0.12, 0.075, 0.045), // Chernozem
        vec3<f32>(0.37, 0.23, 0.12), // Forest soil
        vec3<f32>(0.64, 0.20, 0.075), // Laterite
        vec3<f32>(0.20, 0.18, 0.18), // Volcanic soil
        vec3<f32>(0.58, 0.39, 0.16), // Alluvial soil
        vec3<f32>(0.10, 0.065, 0.045), // Peat
        vec3<f32>(0.72, 0.70, 0.55), // Saline soil
    );
    return palette[index];
}

fn biome_color(kind: f32) -> vec3<f32> {
    let index = min(u32(max(round(kind), 0.0)), 12u);
    let palette = array<vec3<f32>, 13>(
        vec3<f32>(0.075, 0.25, 0.34), // Ocean
        vec3<f32>(0.78, 0.88, 0.91), // Ice
        vec3<f32>(0.43, 0.50, 0.43), // Tundra
        vec3<f32>(0.075, 0.27, 0.20), // Boreal forest
        vec3<f32>(0.10, 0.40, 0.18), // Temperate forest
        vec3<f32>(0.54, 0.62, 0.18), // Temperate grassland
        vec3<f32>(0.42, 0.46, 0.14), // Mediterranean scrub
        vec3<f32>(0.82, 0.58, 0.22), // Desert
        vec3<f32>(0.62, 0.54, 0.13), // Savanna
        vec3<f32>(0.25, 0.42, 0.10), // Tropical seasonal forest
        vec3<f32>(0.025, 0.32, 0.13), // Tropical rainforest
        vec3<f32>(0.38, 0.41, 0.36), // Alpine
        vec3<f32>(0.08, 0.40, 0.34), // Wetland
    );
    return palette[index];
}

fn rock_color(kind: f32) -> vec3<f32> {
    let index = min(u32(max(round(kind), 0.0)), 7u);
    let palette = array<vec3<f32>, 8>(
        vec3<f32>(0.12, 0.18, 0.24), // Oceanic basalt
        vec3<f32>(0.69, 0.36, 0.34), // Felsic craton
        vec3<f32>(0.73, 0.49, 0.20), // Sedimentary rock
        vec3<f32>(0.31, 0.19, 0.17), // Volcanic arc
        vec3<f32>(0.57, 0.47, 0.55), // Plutonic rock
        vec3<f32>(0.47, 0.21, 0.52), // Metamorphic rock
        vec3<f32>(0.76, 0.72, 0.43), // Carbonate platform
        vec3<f32>(0.64, 0.43, 0.18), // Unconsolidated sediment
    );
    return palette[index];
}

fn resource_color(kind: f32) -> vec3<f32> {
    let index = min(u32(max(round(kind), 0.0)), 15u);
    let palette = array<vec3<f32>, 16>(
        vec3<f32>(0.42, 0.43, 0.41), // No dominant deposit
        vec3<f32>(0.70, 0.20, 0.13), // Banded iron formation
        vec3<f32>(0.88, 0.52, 0.22), // Bauxite
        vec3<f32>(0.18, 0.67, 0.59), // Porphyry copper
        vec3<f32>(0.15, 0.52, 0.69), // Volcanogenic massive sulfide
        vec3<f32>(0.43, 0.70, 0.58), // Nickel sulfide
        vec3<f32>(0.95, 0.76, 0.20), // Gold
        vec3<f32>(0.30, 0.69, 0.83), // Gemstones
        vec3<f32>(0.18, 0.18, 0.19), // Coal
        vec3<f32>(0.30, 0.25, 0.19), // Peat
        vec3<f32>(0.34, 0.22, 0.40), // Petroleum
        vec3<f32>(0.58, 0.39, 0.66), // Natural gas
        vec3<f32>(0.88, 0.83, 0.69), // Rock salt
        vec3<f32>(0.69, 0.55, 0.41), // Clay
        vec3<f32>(0.52, 0.67, 0.39), // Phosphate
        vec3<f32>(0.74, 0.72, 0.42), // Nitrate
    );
    return palette[index];
}

fn temperature_color(temp_c: f32) -> vec3<f32> {
    if (temp_c < -10.0) {
        return mix(
            vec3<f32>(0.78, 0.90, 0.96),
            vec3<f32>(0.17, 0.42, 0.70),
            smoothstep(-35.0, -10.0, temp_c),
        );
    }
    if (temp_c < 12.0) {
        return mix(
            vec3<f32>(0.17, 0.42, 0.70),
            vec3<f32>(0.12, 0.59, 0.43),
            smoothstep(-10.0, 12.0, temp_c),
        );
    }
    if (temp_c < 27.0) {
        return mix(
            vec3<f32>(0.12, 0.59, 0.43),
            vec3<f32>(0.93, 0.58, 0.10),
            smoothstep(12.0, 27.0, temp_c),
        );
    }
    return mix(
        vec3<f32>(0.93, 0.58, 0.10),
        vec3<f32>(0.72, 0.06, 0.08),
        smoothstep(27.0, 45.0, temp_c),
    );
}

fn elevation_color(height_m: f32) -> vec3<f32> {
    if (height_m < 0.0) {
        let depth = clamp(-height_m / 6500.0, 0.0, 1.0);
        return mix(vec3<f32>(0.08, 0.36, 0.43), vec3<f32>(0.025, 0.09, 0.18), depth);
    }
    if (height_m < 450.0) {
        return mix(vec3<f32>(0.27, 0.48, 0.28), vec3<f32>(0.43, 0.52, 0.27), height_m / 450.0);
    }
    if (height_m < 2200.0) {
        return mix(
            vec3<f32>(0.43, 0.52, 0.27),
            vec3<f32>(0.47, 0.35, 0.25),
            (height_m - 450.0) / 1750.0,
        );
    }
    return mix(
        vec3<f32>(0.47, 0.35, 0.25),
        vec3<f32>(0.88, 0.90, 0.88),
        clamp((height_m - 2200.0) / 3000.0, 0.0, 1.0),
    );
}

fn elevation_scale_color(height_m: f32) -> vec3<f32> {
    if (height_m < 0.0) {
        let depth = clamp(-height_m / 6500.0, 0.0, 1.0);
        return mix(vec3<f32>(0.30, 0.76, 0.82), vec3<f32>(0.03, 0.10, 0.28), depth);
    }
    if (height_m < 1200.0) {
        return mix(
            vec3<f32>(0.22, 0.57, 0.31),
            vec3<f32>(0.86, 0.78, 0.28),
            height_m / 1200.0,
        );
    }
    if (height_m < 3000.0) {
        return mix(
            vec3<f32>(0.86, 0.78, 0.28),
            vec3<f32>(0.70, 0.24, 0.18),
            (height_m - 1200.0) / 1800.0,
        );
    }
    return mix(
        vec3<f32>(0.70, 0.24, 0.18),
        vec3<f32>(0.96, 0.96, 0.94),
        clamp((height_m - 3000.0) / 3000.0, 0.0, 1.0),
    );
}

fn slope_color(slope_degrees: f32) -> vec3<f32> {
    let gentle = mix(
        vec3<f32>(0.10, 0.34, 0.30),
        vec3<f32>(0.90, 0.78, 0.24),
        smoothstep(0.05, 2.0, slope_degrees),
    );
    let steep = mix(
        gentle,
        vec3<f32>(0.80, 0.24, 0.16),
        smoothstep(2.0, 10.0, slope_degrees),
    );
    return mix(
        steep,
        vec3<f32>(0.18, 0.17, 0.20),
        smoothstep(10.0, 35.0, slope_degrees),
    );
}

fn terrain_normal(surface: vec3<f32>, metres: vec2<f32>) -> vec3<f32> {
    return normalize(vec3<f32>(-surface.y / metres.x, surface.z / metres.y, 1.0));
}

fn directional_light(normal: vec3<f32>) -> f32 {
    let light = normalize(vec3<f32>(-0.55, -0.45, 0.72));
    return clamp(dot(normal, light), 0.25, 1.0);
}

fn contour_line(height_m: f32, interval_m: f32) -> f32 {
    let interval = max(interval_m, 1.0);
    let phase = abs(fract(height_m / interval + 0.5) - 0.5) * interval;
    let width = max(fwidth(height_m) * 0.75, 0.75);
    return 1.0 - smoothstep(width, width * 2.0, phase);
}

fn lod_color(level: u32) -> vec3<f32> {
    let palette = array<vec3<f32>, 6>(
        vec3<f32>(0.93, 0.24, 0.28),
        vec3<f32>(0.96, 0.63, 0.16),
        vec3<f32>(0.35, 0.78, 0.35),
        vec3<f32>(0.15, 0.72, 0.83),
        vec3<f32>(0.34, 0.43, 0.93),
        vec3<f32>(0.82, 0.35, 0.88),
    );
    return palette[level % 6u];
}

fn tile_border(uv: vec2<f32>, width_px: f32) -> f32 {
    let edge_uv = min(uv, vec2<f32>(1.0) - uv);
    let uv_per_pixel = max(fwidth(uv), vec2<f32>(0.000001));
    let edge_pixels = min(edge_uv.x / uv_per_pixel.x, edge_uv.y / uv_per_pixel.y);
    return 1.0 - smoothstep(width_px, width_px + 1.0, edge_pixels);
}

fn layer_relief(light: f32, strength: f32) -> f32 {
    return mix(1.0, 0.78 + light * 0.30, clamp(strength * 0.65, 0.0, 1.0));
}

fn tectonics_view(
    data: vec4<f32>,
    plate_id: f32,
    relative_height: f32,
    light: f32,
    relief: f32,
) -> vec3<f32> {
    var color = plate_color(plate_id);
    if (relative_height < 0.0) {
        let depth = clamp(-relative_height / 6500.0, 0.0, 1.0);
        let ocean = mix(vec3<f32>(0.09, 0.25, 0.33), vec3<f32>(0.025, 0.08, 0.14), depth);
        color = mix(color, ocean, 0.54);
    }
    color *= layer_relief(light, relief * 0.72);

    let boundary = clamp(data.y, -1.0, 1.0);
    let boundary_strength = smoothstep(0.10, 0.62, abs(boundary));
    let boundary_color = select(
        vec3<f32>(0.05, 0.70, 0.91),
        vec3<f32>(1.0, 0.28, 0.055),
        boundary >= 0.0,
    );
    color = mix(color, boundary_color, boundary_strength * 0.94);

    let uplift = clamp(data.z / 4200.0, -1.0, 1.0);
    color = mix(color, vec3<f32>(0.05, 0.13, 0.22), smoothstep(0.18, 0.92, -uplift) * 0.32);
    color = mix(color, vec3<f32>(0.88, 0.70, 0.31), smoothstep(0.22, 0.94, uplift) * 0.34);

    let volcanism = unit_or_scaled(data.w, 1.0);
    let volcanic_field = smoothstep(0.42, 0.72, volcanism);
    let volcanic_core = smoothstep(0.68, 0.90, volcanism);
    let magma = mix(vec3<f32>(0.63, 0.045, 0.025), vec3<f32>(1.0, 0.72, 0.08), volcanic_core);
    color = mix(color, vec3<f32>(0.10, 0.075, 0.07), volcanic_field * 0.42);
    return mix(color, magma, volcanic_core * 0.96);
}

fn hydrology_view(
    data: vec4<f32>,
    relative_height: f32,
    light: f32,
    relief: f32,
) -> vec3<f32> {
    let discharge = clamp(data.x, 0.0, 1.0);
    let wetness = clamp(data.y, 0.0, 1.0);
    let sediment = 1.0 - exp(-max(data.z, 0.0) / 55.0);
    let erosion = 1.0 - exp(-max(data.w, 0.0) / 140.0);
    let ice = clamp(-data.w, 0.0, 1.0);

    if (relative_height < 0.0) {
        let depth = clamp(-relative_height / 6500.0, 0.0, 1.0);
        var ocean = mix(vec3<f32>(0.055, 0.28, 0.39), vec3<f32>(0.018, 0.075, 0.15), depth);
        ocean = mix(ocean, vec3<f32>(0.49, 0.42, 0.25), sediment * (1.0 - depth) * 0.42);
        return ocean * layer_relief(light, relief * 0.30);
    }

    let terrain = mix(
        vec3<f32>(0.25, 0.30, 0.23),
        elevation_color(relative_height),
        0.20,
    ) * layer_relief(light, relief);
    var color = mix(terrain, vec3<f32>(0.08, 0.34, 0.31), smoothstep(0.36, 0.88, wetness) * 0.48);
    color = mix(color, vec3<f32>(0.62, 0.43, 0.19), sediment * 0.34);
    color = mix(color, vec3<f32>(0.42, 0.12, 0.075), erosion * (1.0 - wetness * 0.55) * 0.38);

    let river = pow(smoothstep(0.30, 0.82, discharge), 1.35);
    let river_core = smoothstep(0.70, 0.96, discharge);
    let river_color = mix(vec3<f32>(0.025, 0.43, 0.70), vec3<f32>(0.66, 0.93, 0.98), river_core);
    color = mix(color, river_color, river * 0.98);
    let glacial = smoothstep(0.04, 0.76, ice);
    let glacial_core = smoothstep(0.52, 0.96, ice);
    let ice_color = mix(vec3<f32>(0.55, 0.78, 0.86), vec3<f32>(0.94, 0.98, 1.0), glacial_core);
    return mix(color, ice_color, glacial * 0.94);
}

fn climate_view(
    data: vec4<f32>,
    screen_position: vec2<f32>,
    relative_height: f32,
    light: f32,
    relief: f32,
) -> vec3<f32> {
    let temp_c = data.x;
    let precipitation = max(data.y, 0.0);
    let humidity = 1.0 - exp(-precipitation / 1250.0);
    let dry = 1.0 - smoothstep(220.0, 1050.0, precipitation);
    let wet = smoothstep(1100.0, 3900.0, precipitation);
    var color = temperature_color(temp_c);
    color = mix(color, vec3<f32>(0.68, 0.43, 0.16), dry * 0.46);
    color = mix(color, vec3<f32>(0.055, 0.29, 0.48), wet * 0.43);
    color = mix(color * 0.74, color * 1.05, humidity);
    if (relative_height < 0.0) {
        let depth = clamp(-relative_height / 6500.0, 0.0, 1.0);
        let ocean = mix(vec3<f32>(0.06, 0.28, 0.40), vec3<f32>(0.025, 0.09, 0.18), depth);
        color = mix(ocean, color, 0.30);
    }
    color *= layer_relief(light, relief * 0.34);

    let wind = data.zw;
    let wind_speed = length(wind);
    if (wind_speed > 0.05) {
        let direction = wind / wind_speed;
        let across = dot(screen_position, vec2<f32>(-direction.y, direction.x));
        let along = dot(screen_position, direction);
        let streamline = 1.0 - smoothstep(0.05, 0.18, abs(sin(across * 0.115)));
        let dash = smoothstep(0.10, 0.75, sin(along * 0.055) * 0.5 + 0.5);
        let wind_alpha = streamline * dash * smoothstep(2.0, 14.0, wind_speed) * 0.13;
        color = mix(color, vec3<f32>(0.82, 0.92, 0.92), wind_alpha);
    }
    return color;
}

fn soil_view(
    data: vec4<f32>,
    kind: f32,
    relative_height: f32,
    light: f32,
    relief: f32,
) -> vec3<f32> {
    if (relative_height < 0.0) {
        let depth = clamp(-relative_height / 6500.0, 0.0, 1.0);
        return mix(vec3<f32>(0.11, 0.30, 0.36), vec3<f32>(0.04, 0.12, 0.20), depth);
    }

    let soil_depth = 1.0 - exp(-max(data.y, 0.0) / 1.25);
    let fertility = clamp(data.z, 0.0, 1.0);
    let organic = clamp(data.w, 0.0, 1.0);
    var color = soil_color(kind);
    color *= mix(0.72, 1.08, soil_depth);
    color = mix(color, vec3<f32>(0.16, 0.38, 0.11), fertility * 0.27);
    color = mix(color, vec3<f32>(0.075, 0.055, 0.035), organic * 0.42);
    let category_edge = smoothstep(0.20, 1.5, fwidth(kind));
    color = mix(color, color * 0.54, category_edge);
    return color * layer_relief(light, relief);
}

fn vegetation_view(
    data: vec4<f32>,
    kind: f32,
    relative_height: f32,
    light: f32,
    relief: f32,
) -> vec3<f32> {
    if (relative_height < 0.0) {
        let depth = clamp(-relative_height / 6500.0, 0.0, 1.0);
        return mix(vec3<f32>(0.12, 0.34, 0.41), vec3<f32>(0.035, 0.13, 0.22), depth);
    }

    let tree_cover = clamp(data.y, 0.0, 1.0);
    let grass_cover = clamp(data.z, 0.0, 1.0);
    let productivity = clamp(data.w, 0.0, 1.0);
    var color = biome_color(kind);
    color = mix(color, vec3<f32>(0.025, 0.22, 0.09), tree_cover * 0.48);
    color = mix(color, vec3<f32>(0.57, 0.63, 0.12), grass_cover * (1.0 - tree_cover * 0.55) * 0.38);
    color = mix(color * 0.68, color * 1.12, productivity);
    let category_edge = smoothstep(0.20, 1.5, fwidth(kind));
    color = mix(color, color * 0.58, category_edge);
    return color * layer_relief(light, relief);
}

fn geology_view(
    data: vec4<f32>,
    kind: f32,
    relative_height: f32,
    light: f32,
    relief: f32,
) -> vec3<f32> {
    let age = clamp(data.y / 4500.0, 0.0, 1.0);
    let sediment = 1.0 - exp(-max(data.z, 0.0) / 48.0);
    let ash = 1.0 - exp(-max(data.w, 0.0) / 3.5);
    var color = rock_color(kind);
    color = mix(color * 1.05, color * 0.72 + vec3<f32>(0.055, 0.045, 0.05), age * 0.34);
    color = mix(color, vec3<f32>(0.69, 0.48, 0.20), sediment * 0.34);
    color = mix(color, vec3<f32>(0.33, 0.35, 0.38), ash * 0.46);
    if (relative_height < 0.0) {
        let depth = clamp(-relative_height / 6500.0, 0.0, 1.0);
        let ocean = mix(vec3<f32>(0.07, 0.25, 0.31), vec3<f32>(0.025, 0.08, 0.14), depth);
        color = mix(color, ocean, 0.47);
    }
    let category_edge = smoothstep(0.20, 1.5, fwidth(kind));
    color = mix(color, color * 0.50, category_edge);
    return color * layer_relief(light, relief);
}

fn resources_view(
    data: vec4<f32>,
    kind: f32,
    screen_position: vec2<f32>,
    relative_height: f32,
    light: f32,
    relief: f32,
) -> vec3<f32> {
    var base = mix(vec3<f32>(0.24, 0.27, 0.25), elevation_color(relative_height), 0.24);
    if (relative_height < 0.0) {
        let depth_m = clamp(-relative_height / 6500.0, 0.0, 1.0);
        base = mix(vec3<f32>(0.065, 0.24, 0.31), vec3<f32>(0.02, 0.075, 0.14), depth_m);
    }
    base *= layer_relief(light, relief * 0.80);
    let richness = clamp(data.y, 0.0, 1.0);
    let depth = clamp(data.z, 0.0, 1.0);
    let confidence = clamp(data.w, 0.0, 1.0);
    let has_deposit = select(0.0, 1.0, round(kind) > 0.0);
    let visibility = pow(smoothstep(0.36, 0.90, richness), 1.5) *
        smoothstep(0.24, 0.82, confidence) * mix(1.0, 0.62, depth) * has_deposit;
    var deposit = resource_color(kind);
    deposit = mix(deposit * 0.72, deposit * 1.10, confidence);
    deposit = mix(deposit, vec3<f32>(0.96, 0.90, 0.63), pow(richness, 2.2) * 0.18);

    let marker_cell = fract(screen_position / 11.0) - vec2<f32>(0.5);
    let marker = 1.0 - smoothstep(0.15, 0.31, length(marker_cell));
    let field_tint = visibility * 0.14;
    let marker_tint = visibility * marker * mix(0.82, 0.48, depth);
    return mix(base, deposit, clamp(field_tint + marker_tint, 0.0, 0.92));
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let surface = sample_surface(mesh.uv);
    let metres = max(params.metrics.xy, vec2<f32>(0.01));
    let normal = terrain_normal(surface, metres);
    let light = directional_light(normal);
    let relative_height = surface.x - params.display.y;
    let mode = u32(params.display.x);
    let relief_strength = params.display.w;
    let layer = sample_layer(mesh.uv);
    let layer_kind = sample_layer_kind(mesh.uv);
    var color: vec3<f32>;

    if (mode == 1u) {
        color = elevation_scale_color(relative_height);
    } else if (mode == 2u) {
        let relief = mix(0.5, 0.12 + light * 0.88, relief_strength);
        color = vec3<f32>(relief);
    } else if (mode == 3u) {
        let rise_over_run = length(vec2<f32>(surface.y / metres.x, surface.z / metres.y));
        let slope_degrees = atan(rise_over_run) * 57.2957795;
        color = slope_color(slope_degrees);
    } else if (mode == 4u) {
        color = elevation_color(relative_height);
        let interval = max(params.display.z, 1.0);
        let minor = contour_line(relative_height, interval);
        let major = contour_line(relative_height, interval * 5.0);
        let line = clamp(minor * 0.36 + major * 0.72, 0.0, 1.0);
        color = mix(color, vec3<f32>(0.045, 0.050, 0.055), line);
    } else if (mode == 5u) {
        color = tectonics_view(layer, layer_kind, relative_height, light, relief_strength);
    } else if (mode == 6u) {
        color = hydrology_view(layer, relative_height, light, relief_strength);
    } else if (mode == 7u) {
        color = climate_view(layer, mesh.position.xy, relative_height, light, relief_strength);
    } else if (mode == 8u) {
        color = soil_view(layer, layer_kind, relative_height, light, relief_strength);
    } else if (mode == 9u) {
        color = vegetation_view(layer, layer_kind, relative_height, light, relief_strength);
    } else if (mode == 10u) {
        color = geology_view(layer, layer_kind, relative_height, light, relief_strength);
    } else if (mode == 11u) {
        color = resources_view(
            layer,
            layer_kind,
            mesh.position.xy,
            relative_height,
            light,
            relief_strength,
        );
    } else {
        let illumination = mix(1.0, 0.62 + light * 0.45, relief_strength);
        color = elevation_color(relative_height) * illumination;
    }

    let noise_size = vec2<i32>(textureDimensions(blue_noise_texture, 0));
    let pixel = vec2<i32>(mesh.position.xy);
    let noise_pixel = ((pixel % noise_size) + noise_size) % noise_size;
    let dither = textureLoad(blue_noise_texture, noise_pixel, 0).r - 0.5;
    color += dither * params.metrics.z / 255.0;

    let flags = u32(params.debug.x);
    let desired_level = u32(params.debug.z);
    let source_level = u32(params.debug.w);
    if ((flags & 2u) != 0u) {
        color = mix(color, lod_color(desired_level), 0.30);
    }
    if ((flags & 4u) != 0u) {
        var residency_color = vec3<f32>(0.15, 0.88, 0.47);
        if (source_level < desired_level) {
            residency_color = vec3<f32>(1.0, 0.58, 0.10);
        }
        if ((flags & 8u) != 0u) {
            residency_color = vec3<f32>(1.0, 0.16, 0.72);
        }
        color = mix(color, residency_color, 0.38);
    }
    if ((flags & 1u) != 0u) {
        let border = tile_border(mesh.uv, max(params.debug.y, 0.5));
        color = mix(color, vec3<f32>(0.95), border * 0.9);
    }
    return vec4<f32>(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}

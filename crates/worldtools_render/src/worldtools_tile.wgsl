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

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let surface = sample_surface(mesh.uv);
    let metres = max(params.metrics.xy, vec2<f32>(0.01));
    let normal = terrain_normal(surface, metres);
    let light = directional_light(normal);
    let relative_height = surface.x - params.display.y;
    let mode = u32(params.display.x);
    let relief_strength = params.display.w;
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

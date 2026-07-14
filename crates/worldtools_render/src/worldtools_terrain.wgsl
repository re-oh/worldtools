#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct TerrainMaterialParams {
    view: vec4<f32>,
    display: vec4<f32>,
    style: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> params: TerrainMaterialParams;
@group(#{MATERIAL_BIND_GROUP}) @binding(1)
var elevation_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2)
var blue_noise_texture: texture_2d<f32>;

fn wrap_x(x: i32, width: i32) -> i32 {
    return ((x % width) + width) % width;
}

fn load_height(pixel: vec2<i32>, size: vec2<i32>, period_x: i32) -> f32 {
    let safe = vec2<i32>(wrap_x(pixel.x, period_x), clamp(pixel.y, 0, size.y - 1));
    return textureLoad(elevation_texture, safe, 0).r;
}

fn catmull_rom(a: f32, b: f32, c: f32, d: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    return 0.5 * ((2.0 * b) + (-a + c) * t +
        (2.0 * a - 5.0 * b + 4.0 * c - d) * t2 +
        (-a + 3.0 * b - 3.0 * c + d) * t3);
}

fn catmull_rom_derivative(a: f32, b: f32, c: f32, d: f32, t: f32) -> f32 {
    let t2 = t * t;
    return 0.5 * ((-a + c) +
        2.0 * (2.0 * a - 5.0 * b + 4.0 * c - d) * t +
        3.0 * (-a + 3.0 * b - 3.0 * c + d) * t2);
}

struct ContinuousSample {
    height_m: f32,
    gradient: vec2<f32>,
}

fn continuous_sample(uv: vec2<f32>) -> ContinuousSample {
    let size_u = textureDimensions(elevation_texture, 0);
    let size = vec2<i32>(size_u);
    // Height fields contain both longitudinal seam endpoints. The final
    // column duplicates the first, so interpolation wraps over width - 1.
    let period_x = max(size.x - 1, 1);
    let point = uv * vec2<f32>(f32(period_x), f32(size.y - 1));
    let base = vec2<i32>(floor(point));
    let fraction = fract(point);
    var footprint: array<vec4<f32>, 4>;
    var row_values: vec4<f32>;
    var row_derivatives: vec4<f32>;

    for (var row = 0i; row < 4i; row += 1i) {
        let y = base.y + row - 1i;
        let samples = vec4<f32>(
            load_height(vec2<i32>(base.x - 1i, y), size, period_x),
            load_height(vec2<i32>(base.x, y), size, period_x),
            load_height(vec2<i32>(base.x + 1i, y), size, period_x),
            load_height(vec2<i32>(base.x + 2i, y), size, period_x),
        );
        footprint[u32(row)] = samples;
        row_values[u32(row)] = catmull_rom(
            samples.x, samples.y, samples.z, samples.w, fraction.x,
        );
        row_derivatives[u32(row)] = catmull_rom_derivative(
            samples.x, samples.y, samples.z, samples.w, fraction.x,
        );
    }

    let cubic = catmull_rom(
        row_values.x, row_values.y, row_values.z, row_values.w, fraction.y,
    );
    let gradient = vec2<f32>(
        catmull_rom(
            row_derivatives.x,
            row_derivatives.y,
            row_derivatives.z,
            row_derivatives.w,
            fraction.y,
        ),
        catmull_rom_derivative(
            row_values.x, row_values.y, row_values.z, row_values.w, fraction.y,
        ),
    );
    let local_min = min(
        min(footprint[1].y, footprint[1].z),
        min(footprint[2].y, footprint[2].z),
    );
    let local_max = max(
        max(footprint[1].y, footprint[1].z),
        max(footprint[2].y, footprint[2].z),
    );
    return ContinuousSample(clamp(cubic, local_min, local_max), gradient);
}

fn nearest_height(uv: vec2<f32>) -> f32 {
    let size_u = textureDimensions(elevation_texture, 0);
    let size = vec2<i32>(size_u);
    let period_x = max(size.x - 1, 1);
    let pixel = vec2<i32>(round(uv * vec2<f32>(f32(period_x), f32(size.y - 1))));
    return load_height(pixel, size, period_x);
}

fn elevation_color(height_m: f32) -> vec3<f32> {
    if (height_m < 0.0) {
        let depth = clamp(-height_m / 6500.0, 0.0, 1.0);
        return mix(vec3<f32>(0.08, 0.36, 0.43), vec3<f32>(0.025, 0.09, 0.18), depth);
    }
    if (height_m < 450.0) {
        return mix(
            vec3<f32>(0.27, 0.48, 0.28),
            vec3<f32>(0.43, 0.52, 0.27),
            height_m / 450.0,
        );
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

fn categorical_color(value: f32) -> vec3<f32> {
    let category = i32(round(value));
    switch category % 8i {
        case 0i: { return vec3<f32>(0.10, 0.29, 0.36); }
        case 1i: { return vec3<f32>(0.24, 0.45, 0.27); }
        case 2i: { return vec3<f32>(0.57, 0.55, 0.28); }
        case 3i: { return vec3<f32>(0.49, 0.35, 0.24); }
        case 4i: { return vec3<f32>(0.56, 0.32, 0.38); }
        case 5i: { return vec3<f32>(0.27, 0.48, 0.54); }
        case 6i: { return vec3<f32>(0.54, 0.49, 0.43); }
        default: { return vec3<f32>(0.75, 0.76, 0.72); }
    }
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let centered = mesh.uv - vec2<f32>(0.5);
    let world_uv = vec2<f32>(
        fract(params.view.x + centered.x * params.view.z),
        clamp(params.view.y + centered.y * params.view.w, 0.0, 1.0),
    );
    let categorical = params.style.z > 0.5;
    if (categorical) {
        let category = nearest_height(world_uv);
        return vec4<f32>(categorical_color(category), 1.0);
    }

    let terrain = continuous_sample(world_uv);
    var color = elevation_color(terrain.height_m);
    let normal = normalize(vec3<f32>(
        -terrain.gradient.x * 0.003,
        terrain.gradient.y * 0.003,
        1.0,
    ));
    let light = normalize(vec3<f32>(-0.55, -0.45, 0.72));
    let shade = clamp(dot(normal, light), 0.35, 1.0);
    color *= 0.72 + shade * 0.38;

    let noise_size = vec2<i32>(textureDimensions(blue_noise_texture, 0));
    let pixel = vec2<i32>(mesh.position.xy);
    let noise_pixel = vec2<i32>(wrap_x(pixel.x, noise_size.x), wrap_x(pixel.y, noise_size.y));
    let dither = textureLoad(blue_noise_texture, noise_pixel, 0).r - 0.5;
    color += dither * params.display.w / 255.0;
    return vec4<f32>(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}

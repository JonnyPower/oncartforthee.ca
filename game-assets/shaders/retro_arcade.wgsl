// Retro Arcade Shader (Pixelation + Yellow Tint)
@group(0) @binding(0) var textureSampler: sampler;
@group(0) @binding(1) var textureData: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>
};

@vertex
fn vertex_main(@location(0) position: vec2<f32>, @location(1) uv: vec2<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(position, 0.0, 1.0);
    out.uv = uv;
    return out;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Pixelation parameters
    let pixelSize: vec2<f32> = vec2<f32>(5.0, 5.0); // Adjust for more or less pixelation
    let screenRes: vec2<f32> = vec2<f32>(800.0, 600.0); // Adjust based on actual screen size
    let uv_pixelated: vec2<f32> = floor(in.uv * screenRes / pixelSize) * pixelSize / screenRes;
    
    // Sample the texture with pixelated UVs
    let color: vec4<f32> = textureSample(textureData, textureSampler, uv_pixelated);
    
    // Apply a slight yellow tint
    let yellowTint: vec3<f32> = vec3<f32>(1.2, 1.1, 0.8); // Warmer tones
    let finalColor: vec3<f32> = color.rgb * yellowTint;
    
    return vec4<f32>(finalColor, color.a);
}

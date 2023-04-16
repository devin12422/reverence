
struct Out {
    @builtin(position) position: vec4<f32>,
    @location(0) cell:f32,
};

struct LifeParams {
    width : u32,
    height : u32,
    threshold : f32,
};
@group(0) @binding(0) var<uniform> params : LifeParams;
@vertex
fn vs_main(@builtin(instance_index) i:u32,
    @location(0) cell:u32,
    @location(1) pos: vec2<u32>,
) -> Out {
  let x = (f32(i % params.width + pos.x) / f32(params.width) - 0.5) * 2. * f32(params.width) / f32(max(params.width, params.height));
  let y = (f32((i - (i % params.width)) / params.width + pos.y) / f32(params.height) - 0.5) * 2. * f32(params.height) / f32(max(params.width, params.height));

  return Out(vec4<f32>(x, y, 0., 1.), f32(cell));
}

@fragment
fn fs_main(@location(0) cell:f32) -> @location(0) vec4<f32> {
    return vec4<f32>(cell,cell,cell,1.);    
}

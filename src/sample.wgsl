// todo: put uniforms first?
@group(0)
@binding(0)
var t: texture_2d<f32>;

@group(0)
@binding(1)
var s: sampler;

@group(0)
@binding(2)
var<uniform> resolution: f32;

const VERTICES = array(
  vec4(-1.0, -1.0, 0.0, 1.0),
  vec4(3.0, -1.0, 0.0, 1.0),
  vec4(-1.0, 3.0, 0.0, 1.0)
);

fn quadrant(position: vec2<f32>) -> vec2<f32> {
  return (position + 1.0) / 2.0;
}

@vertex
fn vertex(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  return VERTICES[i];
}

@fragment
fn fragment(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let uv = quadrant(position.xy / resolution * 2.0 - 1.0);
  let color = textureSample(t, s, uv);
  if min(abs((1.0 - uv.x) - uv.y), abs(uv.x - uv.y)) < 0.1 {
    return vec4((color.xyx - 1) * -1, 1.0);
  } else {
    return color;
  }
}

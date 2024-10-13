const VERTICES = array(
  vec4(-1.0, -1.0, 0.0, 1.0),
  vec4(3.0, -1.0, 0.0, 1.0),
  vec4(-1.0, 3.0, 0.0, 1.0)
);

@vertex
fn vertex(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  return VERTICES[i];
}

@fragment
fn fragment() -> @location(0) vec4<f32> {
  return vec4<f32>(1, 1, 1, 1);
}

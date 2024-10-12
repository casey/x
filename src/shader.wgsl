// const TRI_VERTICES = array(
//   vec4(0., 0., 0., 1.),
//   vec4(0., 1., 0., 1.),
//   vec4(1., 1., 0., 1.),
// );

@vertex
fn vertex(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let x = f32(i32(i) - 1);
  let y = f32(i32(i & 1) * 2 - 1);
  return vec4<f32>(x, y, 0, 1);
}

@fragment
fn fragment() -> @location(0) vec4<f32> {
  return vec4<f32>(1, 1, 1, 1);
}

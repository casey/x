@group(0)
@binding(0)
var<uniform> uniforms: Uniforms;

@group(0)
@binding(1)
var source: texture_2d<f32>;

@group(0)
@binding(2)
var source_sampler: sampler;

const ERROR_COLOR = vec4(0.0, 1.0, 0.0, 1.0);

const FIELD_ALL: u32 = 0;
const FIELD_NONE: u32 = 1;
const FIELD_X: u32 = 2;

const VERTICES = array(
  vec4(-1.0, -1.0, 0.0, 1.0),
  vec4(3.0, -1.0, 0.0, 1.0),
  vec4(-1.0, 3.0, 0.0, 1.0)
);

struct Uniforms {
  field: u32,
  resolution: f32,
}

fn field_all(uv: vec2<f32>) -> bool {
  return true;
}

fn field_none(uv: vec2<f32>) -> bool {
  return false;
}

fn field_x(uv: vec2<f32>) -> bool {
  return min(abs((1.0 - uv.x) - uv.y), abs(uv.x - uv.y)) < 0.1;
}

fn invert(color: vec4<f32>) -> vec4<f32> {
  return vec4((color.xyx - 1) * -1, 1.0);
}

@vertex
fn vertex(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  return VERTICES[i];
}

@fragment
fn fragment(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let uv = position.xy / uniforms.resolution;
  let input = textureSample(source, source_sampler, uv);
  var on: bool;
  switch uniforms.field {
    case FIELD_ALL {
      on = field_all(uv);
    }
    case FIELD_NONE {
      on = field_none(uv);
    }
    case FIELD_X {
      on = field_x(uv);
    }
    default {
      return ERROR_COLOR;
    }
  }
  if on {
    return invert(input);
  } else {
    return input;
  }
}

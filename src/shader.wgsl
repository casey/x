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
const BLACK = vec4(0.0, 0.0, 0.0, 1.0);

const FIELD_ALL: u32 = 0;
const FIELD_CIRCLE: u32 = 1;
const FIELD_NONE: u32 = 2;
const FIELD_X: u32 = 3;

const FALSE: u32 = 0;
const TRUE: u32 = 1;

const VERTICES = array(
  vec4(-1.0, -1.0, 0.0, 1.0),
  vec4(3.0, -1.0, 0.0, 1.0),
  vec4(-1.0, 3.0, 0.0, 1.0)
);

struct Uniforms {
  field: u32,
  fit: u32,
  repeat: u32,
  resolution: vec2f,
}

fn field_all(p: vec2f) -> bool {
  return true;
}

fn field_circle(p: vec2f) -> bool {
  return length(p) < 1;
}

fn field_none(p: vec2f) -> bool {
  return false;
}

fn field_x(p: vec2f) -> bool {
  return abs(abs(p.x) - abs(p.y)) < 0.2;
}

fn invert(color: vec4f) -> vec4f {
  return vec4((color.xyx - 1) * -1, 1);
}

@vertex
fn vertex(@builtin(vertex_index) i: u32) -> @builtin(position) vec4f {
  return VERTICES[i];
}

@fragment
fn fragment(@builtin(position) position: vec4f) -> @location(0) vec4f {
  let quad = position.xy / uniforms.resolution;

  var centered = quad - 0.5;

  let aspect = uniforms.resolution.x / uniforms.resolution.y;

  if uniforms.fit == TRUE {
    if aspect > 1 {
      centered.x *= aspect;
    } else {
      centered.y /= aspect;
    }
  } else {
    if aspect > 1 {
      centered.y /= aspect;
    } else {
      centered.x *= aspect;
    }
  }

  let uv = centered + 0.5;

  var input = BLACK;

  if uniforms.repeat == TRUE || (all(uv >= vec2(0.0, 0.0)) && all(uv <= vec2(1.0, 1.0))) {
    input = textureSample(source, source_sampler, uv);
  }

  let p = uv * 2 - 1;;
  var on: bool;
  switch uniforms.field {
    case FIELD_ALL {
      on = field_all(p);
    }
    case FIELD_CIRCLE {
      on = field_circle(p);
    }
    case FIELD_NONE {
      on = field_none(p);
    }
    case FIELD_X {
      on = field_x(p);
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

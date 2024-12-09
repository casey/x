@group(0)
@binding(0)
var<uniform> uniforms: Uniforms;

@group(0)
@binding(1)
var image: texture_2d<f32>;

@group(0)
@binding(2)
var source: texture_2d<f32>;

// rename to texture_sampler
@group(0)
@binding(3)
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
  color: mat4x4f,
  coordinates: u32,
  field: u32,
  filters: u32,
  fit: u32,
  image_alpha: f32,
  index: u32,
  offset: vec2f,
  position: mat3x3f,
  repeat: u32,
  resolution: vec2f,
  source_alpha: f32,
  source_offset: vec2f,
  tiling: u32,
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
  // if uniforms.index < uniforms.filters {
  //   let color = (1 / f32(uniforms.filters)) * f32(uniforms.index);

  //   return vec4(color, color, color, 1.0);
  // }

  // subtract offset get tile coordinates
  let tile = position.xy - uniforms.offset;

  // convert fragment coordinates to [-1, 1]
  let centered = tile / uniforms.resolution * 2 - 1;

  // apply position transform
  var transformed = (uniforms.position * vec3(centered, 1)).xy;

  // calculate aspect ratio
  let aspect = uniforms.resolution.x / uniforms.resolution.y;

  if uniforms.fit == TRUE {
    // fit to viewport
    if aspect > 1 {
      transformed.x *= aspect;
    } else {
      transformed.y /= aspect;
    }
  } else {
    // fill viewport
    if aspect > 1 {
      transformed.y /= aspect;
    } else {
      transformed.x *= aspect;
    }
  }

  // convert position to uv coordinates
  let uv = (transformed + 1) / 2;

  var input = BLACK;

  // todo: if using coordinate colors, figure out how to blend in
  if uniforms.coordinates == TRUE {
    input = vec4(uv, 1.0, 1.0);
  } else if uniforms.repeat == TRUE || (all(uv >= vec2(0.0, 0.0)) && all(uv <= vec2(1.0, 1.0))) {
    // 0..1
    // 0..0.5
    // convert uv coordinates to tile source coordinates
    let tile_uv = uv / f32(uniforms.tiling) + uniforms.source_offset;
    // read the input color
    input = textureSample(source, source_sampler, tile_uv);
  }

  let image_input = textureSample(image, source_sampler, uv);

  // todo: do real alpha blending
  input = input * uniforms.source_alpha + image_input * uniforms.image_alpha;

  var on: bool;

  switch uniforms.field {
    case FIELD_ALL {
      on = field_all(transformed);
    }
    case FIELD_CIRCLE {
      on = field_circle(transformed);
    }
    case FIELD_NONE {
      on = field_none(transformed);
    }
    case FIELD_X {
      on = field_x(transformed);
    }
    default {
      return ERROR_COLOR;
    }
  }

  if on {
    // convert rgb color to [-1, 1]
    let centered = input * 2 - 1;

    // apply color transform
    let transformed = uniforms.color * centered;

    // convert back to rgb
    let color = (transformed + 1) / 2;

    return color;
  } else {
    return input;
  }
}

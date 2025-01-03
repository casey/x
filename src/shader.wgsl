@group(0)
@binding(0)
var filtering_sampler: sampler;

@group(0)
@binding(1)
var image: texture_2d<f32>;

@group(0)
@binding(2)
var non_filtering_sampler: sampler;

@group(0)
@binding(3)
var samples: texture_1d<f32>;

@group(0)
@binding(4)
var source: texture_2d<f32>;

@group(0)
@binding(5)
var<uniform> uniforms: Uniforms;

const ERROR_COLOR = vec4(0.0, 1.0, 0.0, 1.0);
const BLACK = vec4(0.0, 0.0, 0.0, 1.0);

const FIELD_ALL: u32 = 0;
const FIELD_CIRCLE: u32 = 1;
const FIELD_NONE: u32 = 2;
const FIELD_SAMPLES: u32 = 3;
const FIELD_X: u32 = 4;

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
  image_read: u32,
  index: u32,
  offset: vec2f,
  position: mat3x3f,
  repeat: u32,
  resolution: vec2f,
  sample_range: f32,
  source_offset: vec2f,
  source_read: u32,
  spl: f32,
  tiling: u32,
  wrap: u32,
}

fn field_all(p: vec2f) -> bool {
  return true;
}

fn field_circle(p: vec2f) -> bool {
  return length(p) < 1 + uniforms.spl;
}

fn field_none(p: vec2f) -> bool {
  return false;
}

fn field_samples(p: vec2f) -> bool {
  let x = (p.x + 1) * 0.5 * uniforms.sample_range;
  let sample = textureSample(samples, non_filtering_sampler, x).x;
  return sample * 4 < p.y;
}

fn field_x(p: vec2f) -> bool {
  return abs(abs(p.x) - abs(p.y)) < 0.2 + uniforms.spl;
}

fn invert(color: vec4f) -> vec4f {
  return vec4((color.xyx - 1) * -1, 1);
}

fn read(uv: vec2f) -> bool {
  return bool(uniforms.repeat) || all(uv >= vec2(0.0, 0.0)) && all(uv <= vec2(1.0, 1.0));
}

@vertex
fn vertex(@builtin(vertex_index) i: u32) -> @builtin(position) vec4f {
  return VERTICES[i];
}

@fragment
fn fragment(@builtin(position) position: vec4f) -> @location(0) vec4f {
  // subtract offset get tile coordinates
  let tile = position.xy - uniforms.offset;

  // convert fragment coordinates to [-1, 1]
  let centered = tile / uniforms.resolution * 2 - 1;

  // apply position transform
  var transformed = (uniforms.position * vec3(centered, 1)).xy;

  // calculate aspect ratio
  let aspect = uniforms.resolution.x / uniforms.resolution.y;

  if bool(uniforms.fit) {
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
  var uv = (transformed + 1) / 2;

  // wrap uv coordinates
  if bool(uniforms.wrap) {
    uv = fract(uv);
  }

  var input = BLACK;

  if bool(uniforms.coordinates) {
    input = vec4(uv, 1.0, 1.0);
  } else if bool(uniforms.source_read) && read(uv) {
    // convert uv coordinates to tile source coordinates
    var tile_uv = uv / f32(uniforms.tiling) + uniforms.source_offset;

    if uniforms.index < uniforms.filters {
      // scale to compensate for tiles not taking up full source texture
      let scale = uniforms.resolution * f32(uniforms.tiling)
        / vec2f(textureDimensions(source, 0));
      tile_uv *= scale;
    }

    // read the input color
    input = textureSample(source, filtering_sampler, tile_uv);
  }

  if bool(uniforms.image_read) && read(uv) {
    input += textureSample(image, filtering_sampler, uv);
  }

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
    case FIELD_SAMPLES {
      on = field_samples(transformed);
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

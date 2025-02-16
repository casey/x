@group(0)
@binding(0)
var back: texture_2d<f32>;

@group(0)
@binding(1)
var filtering_sampler: sampler;

@group(0)
@binding(2)
var frequencies: texture_1d<f32>;

@group(0)
@binding(3)
var front: texture_2d<f32>;

@group(0)
@binding(4)
var non_filtering_sampler: sampler;

@group(0)
@binding(5)
var samples: texture_1d<f32>;

@group(0)
@binding(6)
var<uniform> uniforms: Uniforms;

const ERROR = vec4(0.0, 1.0, 0.0, 1.0);
const TRANSPARENT = vec4(0.0, 0.0, 0.0, 0.0);

const FIELD_ALL: u32 = 0;
const FIELD_CIRCLE: u32 = 1;
const FIELD_FREQUENCIES: u32 = 2;
const FIELD_NONE: u32 = 3;
const FIELD_SAMPLES: u32 = 4;
const FIELD_TOP: u32 = 5;
const FIELD_X: u32 = 6;

const VERTICES = array(
  vec4(-1.0, -1.0, 0.0, 1.0),
  vec4(3.0, -1.0, 0.0, 1.0),
  vec4(-1.0, 3.0, 0.0, 1.0)
);

struct Uniforms {
  back_read: u32,
  color: mat4x4f,
  coordinates: u32,
  field: u32,
  filters: u32,
  fit: u32,
  frequency_range: f32,
  front_offset: vec2f,
  front_read: u32,
  index: u32,
  offset: vec2f,
  position: mat3x3f,
  repeat: u32,
  resolution: vec2f,
  sample_range: f32,
  tiling: u32,
  wrap: u32,
}

fn field_all(p: vec2f) -> bool {
  return true;
}

fn field_circle(p: vec2f) -> bool {
  return length(p) < 1;
}

fn field_frequencies(p: vec2f) -> bool {
  let x = (p.x + 1) * 0.5 * uniforms.frequency_range;
  let level = textureSample(frequencies, non_filtering_sampler, x).x;
  return level > (-p.y + 1) * 0.5;
}

fn field_none(p: vec2f) -> bool {
  return false;
}

fn field_samples(p: vec2f) -> bool {
  let x = (p.x + 1) * 0.5 * uniforms.sample_range;
  let level = textureSample(samples, non_filtering_sampler, x).x;
  return level < p.y;
}

fn field_top(p: vec2f) -> bool {
  return p.y < 0;
}

fn field_x(p: vec2f) -> bool {
  return abs(abs(p.x) - abs(p.y)) < 0.2;
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

  var front_color = TRANSPARENT;

  if bool(uniforms.front_read) && read(uv) {
    if bool(uniforms.coordinates) {
      front_color = vec4(uv, 1.0, 1.0);
    } else {
      // convert uv coordinates to tile source coordinates
      var tile_uv = uv / f32(uniforms.tiling) + uniforms.front_offset;

      if uniforms.index < uniforms.filters {
        // scale to compensate for tiles not taking up full front texture
        let scale = uniforms.resolution * f32(uniforms.tiling)
          / vec2f(textureDimensions(front, 0));
        tile_uv *= scale;
      }

      // read the input color
      front_color = textureSample(front, filtering_sampler, tile_uv);
    }
  }

  var back_color = TRANSPARENT;

  if bool(uniforms.back_read) && read(uv) {
    back_color = textureSample(back, filtering_sampler, uv);
  }

  let input = vec4(front_color.rgb * front_color.a + back_color.rgb * (1 - front_color.a), 1.0);

  var on: bool;

  switch uniforms.field {
    case FIELD_ALL {
      on = field_all(transformed);
    }
    case FIELD_CIRCLE {
      on = field_circle(transformed);
    }
    case FIELD_FREQUENCIES {
      on = field_frequencies(transformed);
    }
    case FIELD_NONE {
      on = field_none(transformed);
    }
    case FIELD_SAMPLES {
      on = field_samples(transformed);
    }
    case FIELD_TOP {
      on = field_top(transformed);
    }
    case FIELD_X {
      on = field_x(transformed);
    }
    default {
      return ERROR;
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

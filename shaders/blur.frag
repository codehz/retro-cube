#version 450

layout(location = 0) uniform sampler2D color_sample;
layout(binding = 0, std140) uniform block { vec2 direction; };
layout(location = 0) out vec4 color;

vec4 fetch(sampler2D image, vec2 pos, vec2 resolution, vec2 direction) {
  vec2 uv = pos / resolution;
  vec4 color = vec4(0.0);
  for (int i = -20; i <= 20; i++) {
    vec2 off = i * direction / resolution;
    float f = clamp(float(abs(i)) / 100.0, 0.0, 1.0);
    vec4 current = texture2D(image, uv + off);
    color = clamp(color + float(1.0 - current.a > f) * current, color, current);
  }
  return color;
}

void main() {
  color = fetch(color_sample, gl_FragCoord.xy, textureSize(color_sample, 0),
                direction);
}
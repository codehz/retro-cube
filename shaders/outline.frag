#version 450

layout(location = 0) uniform sampler2D color_sample;
layout(location = 1) uniform sampler2D normal_sample;
layout(location = 2) uniform sampler2D position_sample;

layout(location = 0) out vec4 color;

vec3 fetchPosition(ivec2 off) {
  vec2 resolution = textureSize(position_sample, 0);
  return texture(position_sample, vec2(gl_FragCoord.xy + off) / resolution).xyz;
}
vec3 fetchNormal(ivec2 off) {
  vec2 resolution = textureSize(position_sample, 0);
  return texture(normal_sample, vec2(gl_FragCoord.xy + off) / resolution).xyz;
}
vec3 fetchColor(ivec2 off) {
  vec2 resolution = textureSize(color_sample, 0);
  return texture(color_sample, vec2(gl_FragCoord.xy + off) / resolution).xyz;
}

float kernel[9] = float[9](
    // clang-format off
    -1, -1, -1,
    -1, 8, -1,
    -1, -1, -1
    // clang-format on
);

void main() {
  vec3 position_diff;
  vec3 normal_diff;
  vec3 color_diff;

  for (int i = 0; i < 3; i++) {
    for (int j = 0; j < 3; j++) {
      int idx = i + j * 3;
      ivec2 off = ivec2(i - 1, j - 1);
      position_diff += fetchPosition(off) * kernel[idx];
      normal_diff += fetchNormal(off) * kernel[idx];
      color_diff += fetchColor(off) * kernel[idx];
    }
  }

  float nd = length(normal_diff);
  float pd = length(position_diff);
  float cd = length(color_diff);
  float score = min(1.0, float(nd + pd + cd > 1.0) + 0.02);

  color = vec4(fetchColor(ivec2(0, 0)) * score, fetchPosition(ivec2(0.0)).z);
}
#version 450

layout(location = 0) uniform sampler2D color_sample;
layout(binding = 0, std140) uniform block { float near, far; };
layout(location = 0) out vec4 color;

#define K 10

float sstep(float x, float z) {
  float rz = 1.0 - clamp((z - near) / (far - near), 0.0, 1.0);
  if (x < rz) {
    return min(1.0, sqrt(x) * pow(rz, 2.0));
  } else if (x < rz * 2.0) {
    return min(0.01, x * rz * 2.0);
  } else {
    return 0;
  }
}

vec4 strengthen(vec2 pos, vec2 unit) {
  vec4 color = vec4(0.0);
  for (int i = -K; i <= K; i++) {
    int dv = int(sqrt(K * K - i * i));
    for (int j = -dv; j <= dv; j++) {
      float len = sqrt(j * j + i * i);
      float f = len / K;
      vec4 current = texture2D(color_sample, pos + vec2(i, j) * unit);
      color = clamp(color + sstep(f, current.a) * current, color,
                    max(color, current));
    }
  }
  return color;
}

void main() {
  vec2 size = textureSize(color_sample, 0);
  color = strengthen(gl_FragCoord.xy / size, 1.0 / size);
}
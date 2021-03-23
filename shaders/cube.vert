#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;
layout(location = 2) in uint face;
layout(location = 0) out vec3 gcolor;
layout(location = 1) out uint gface;

layout(location = 0) uniform mat4 perspective;
layout(location = 1) uniform mat4 view_model;

void main() {
  gcolor = color;
  gface = face;
  gl_Position = vec4(position, 0.0);
}
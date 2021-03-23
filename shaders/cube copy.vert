#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;
layout(location = 2) in vec3 normal;
layout(location = 0) out vec3 v_color;
layout(location = 1) out vec3 v_normal;

layout(location = 0) uniform mat4 perspective;
layout(location = 1) uniform mat4 view_model;

void main() {
  v_color = color;
  v_normal = normal;
  gl_Position = perspective * view_model * vec4(position, 1.0);
}
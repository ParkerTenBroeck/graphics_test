precision mediump float;
                
out vec4 FragColor;
in vec2 uv;

uniform sampler2D tex;

void main() {
    FragColor = texture(tex, uv);
    // FragColor.y *= 0.5;
    // FragColor.z *= 0.5;
}
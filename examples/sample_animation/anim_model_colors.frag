#version 330 core
out vec4 FragColor;

//in vec2 TexCoords;
flat in int fragColorId;

uniform sampler2D texture_diffuse1;

//#define N 10// A smaller example, change to 100 for your actual use case

vec4 rainbowColors[10] = vec4[10](
    vec4(1.0, 0.0, 0.0, 1.0), // Red
    vec4(1.0, 0.5, 0.0, 1.0), // Orange
    vec4(1.0, 1.0, 0.0, 1.0), // Yellow
    vec4(0.5, 1.0, 0.0, 1.0), // Yellow-Green
    vec4(0.0, 1.0, 0.0, 1.0), // Green
    vec4(0.0, 1.0, 0.5, 1.0), // Green-Cyan
    vec4(0.0, 1.0, 1.0, 1.0), // Cyan
    vec4(0.0, 0.5, 1.0, 1.0), // Cyan-Blue
    vec4(0.0, 0.0, 1.0, 1.0), // Blue
    vec4(0.5, 0.0, 1.0, 1.0)  // Violet
);

int modi(int x, int y) {
    return x - (y * (x / y));
}

void main()
{
    int colorid = modi(fragColorId, 10);
    FragColor = rainbowColors[colorid];

//    FragColor = texture(texture_diffuse1, TexCoords);
}

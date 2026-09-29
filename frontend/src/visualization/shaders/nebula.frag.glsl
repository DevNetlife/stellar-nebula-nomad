uniform float uTime;

varying vec3 vColor;
varying float vAlpha;

void main() {
    // Distance from center of point sprite [0, 0.5]
    vec2 coord = gl_PointCoord - vec2(0.5);
    float dist = length(coord);

    if (dist > 0.5) {
        discard;
    }

    // Soft radial falloff for celestial gas effect
    float intensity = exp(-dist * 8.0);
    float glow = 1.0 - smoothstep(0.0, 0.5, dist);

    float alpha = vAlpha * (intensity * 0.7 + glow * 0.3);

    // Core highlight
    vec3 finalColor = mix(vColor, vec3(1.0, 1.0, 1.0), intensity * 0.5);

    gl_FragColor = vec4(finalColor, alpha);
}

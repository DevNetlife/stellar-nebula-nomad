uniform float uTime;
uniform float uPixelRatio;
uniform float uScale;

attribute float aSize;
attribute vec3 aColor;
attribute float aAlpha;

varying vec3 vColor;
varying float vAlpha;

void main() {
    vColor = aColor;
    vAlpha = aAlpha;

    vec3 pos = position;

    // Subtle orbital swirl
    float angle = uTime * 0.05 * (1.0 / (length(pos.xz) + 0.1));
    float cosA = cos(angle);
    float sinA = sin(angle);
    pos.x = position.x * cosA - position.z * sinA;
    pos.z = position.x * sinA + position.z * cosA;

    // Vertical wave pulsation
    pos.y += sin(uTime * 0.5 + length(position)) * 0.08;

    vec4 mvPosition = modelViewMatrix * vec4(pos, 1.0);
    gl_Position = projectionMatrix * mvPosition;

    // Size attenuation based on depth
    gl_PointSize = aSize * uScale * uPixelRatio * (200.0 / -mvPosition.z);
    gl_PointSize = max(gl_PointSize, 1.0);
}

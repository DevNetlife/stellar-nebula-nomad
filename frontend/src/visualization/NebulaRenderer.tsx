import React, { useEffect, useRef, useState, useCallback } from 'react';
import * as THREE from 'three';
import {
  isWebGLAvailable,
  QualityPreset,
  QUALITY_CONFIGS,
  NEBULA_PALETTES,
  NebulaPalette,
  generateNebulaParticles,
  generateCelestialNodes,
  CelestialNode,
  render2DCanvasFallback,
} from './utils';

// Fallback shader strings in case GLSL loaders are not configured in bundler
const DEFAULT_VERT_SHADER = `
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
      float angle = uTime * 0.04 * (1.0 / (length(pos.xz) + 0.1));
      float cosA = cos(angle);
      float sinA = sin(angle);
      pos.x = position.x * cosA - position.z * sinA;
      pos.z = position.x * sinA + position.z * cosA;

      pos.y += sin(uTime * 0.4 + length(position)) * 0.08;

      vec4 mvPosition = modelViewMatrix * vec4(pos, 1.0);
      gl_Position = projectionMatrix * mvPosition;
      gl_PointSize = aSize * uScale * uPixelRatio * (200.0 / -mvPosition.z);
      gl_PointSize = max(gl_PointSize, 1.0);
  }
`;

const DEFAULT_FRAG_SHADER = `
  uniform float uTime;
  varying vec3 vColor;
  varying float vAlpha;

  void main() {
      vec2 coord = gl_PointCoord - vec2(0.5);
      float dist = length(coord);
      if (dist > 0.5) discard;

      float intensity = exp(-dist * 8.0);
      float glow = 1.0 - smoothstep(0.0, 0.5, dist);
      float alpha = vAlpha * (intensity * 0.7 + glow * 0.3);
      vec3 finalColor = mix(vColor, vec3(1.0, 1.0, 1.0), intensity * 0.5);

      gl_FragColor = vec4(finalColor, alpha);
  }
`;

export interface NebulaRendererProps {
  initialQuality?: QualityPreset;
  initialPalette?: string;
  onNodeSelect?: (node: CelestialNode | null) => void;
  width?: string | number;
  height?: string | number;
  className?: string;
}

export const NebulaRenderer: React.FC<NebulaRendererProps> = ({
  initialQuality = 'medium',
  initialPalette = 'orion',
  onNodeSelect,
  width = '100%',
  height = '100%',
  className = '',
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvas2DRef = useRef<HTMLCanvasElement>(null);

  const [quality, setQuality] = useState<QualityPreset>(initialQuality);
  const [selectedPaletteKey, setSelectedPaletteKey] = useState<string>(initialPalette);
  const [selectedNode, setSelectedNode] = useState<CelestialNode | null>(null);
  const [webGLSupported, setWebGLSupported] = useState<boolean>(true);
  const [is2DFallback, setIs2DFallback] = useState<boolean>(false);
  const [fps, setFps] = useState<number>(60);
  const [isLoading, setIsLoading] = useState<boolean>(true);

  // References for animation and clean up
  const sceneRef = useRef<THREE.Scene | null>(null);
  const rendererRef = useRef<THREE.WebGLRenderer | null>(null);
  const cameraRef = useRef<THREE.PerspectiveCamera | null>(null);
  const pointsRef = useRef<THREE.Points | null>(null);
  const starsRef = useRef<THREE.Points | null>(null);
  const nodeMeshesRef = useRef<{ mesh: THREE.Mesh; node: CelestialNode }[]>([]);
  const animFrameIdRef = useRef<number | null>(null);
  const nodes = useRef<CelestialNode[]>(generateCelestialNodes()).current;

  // Orbit control state
  const isDragging = useRef<boolean>(false);
  const previousMousePosition = useRef<{ x: number; y: number }>({ x: 0, y: 0 });
  const cameraSpherical = useRef<{ radius: number; theta: number; phi: number }>({
    radius: 45,
    theta: Math.PI / 4,
    phi: Math.PI / 3,
  });

  const updateCameraPosition = useCallback(() => {
    if (!cameraRef.current) return;
    const { radius, theta, phi } = cameraSpherical.current;
    cameraRef.current.position.x = radius * Math.sin(phi) * Math.cos(theta);
    cameraRef.current.position.y = radius * Math.cos(phi);
    cameraRef.current.position.z = radius * Math.sin(phi) * Math.sin(theta);
    cameraRef.current.lookAt(0, 0, 0);
  }, []);

  // WebGL 3D setup
  useEffect(() => {
    if (!isWebGLAvailable()) {
      setWebGLSupported(false);
      setIs2DFallback(true);
      setIsLoading(false);
      return;
    }

    const container = containerRef.current;
    if (!container) return;

    const w = container.clientWidth || 800;
    const h = container.clientHeight || 600;

    // Scene setup
    const scene = new THREE.Scene();
    scene.fog = new THREE.FogExp2(0x050711, QUALITY_CONFIGS[quality].fogDensity);
    sceneRef.current = scene;

    // Camera setup
    const camera = new THREE.PerspectiveCamera(60, w / h, 0.1, 1000);
    cameraRef.current = camera;
    updateCameraPosition();

    // Renderer setup
    let renderer: THREE.WebGLRenderer;
    try {
      renderer = new THREE.WebGLRenderer({ antialias: quality !== 'low', alpha: true, powerPreference: 'high-performance' });
    } catch {
      setWebGLSupported(false);
      setIs2DFallback(true);
      setIsLoading(false);
      return;
    }

    renderer.setSize(w, h);
    renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, QUALITY_CONFIGS[quality].dprLimit));
    renderer.setClearColor(0x050711, 1);
    container.innerHTML = '';
    container.appendChild(renderer.domElement);
    rendererRef.current = renderer;

    // Nebula Particle System
    const palette = NEBULA_PALETTES[selectedPaletteKey] || NEBULA_PALETTES.orion;
    const particleData = generateNebulaParticles(QUALITY_CONFIGS[quality].particleCount, palette);

    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute('position', new THREE.BufferAttribute(particleData.positions, 3));
    geometry.setAttribute('aColor', new THREE.BufferAttribute(particleData.colors, 3));
    geometry.setAttribute('aSize', new THREE.BufferAttribute(particleData.sizes, 1));
    geometry.setAttribute('aAlpha', new THREE.BufferAttribute(particleData.alphas, 1));

    const material = new THREE.ShaderMaterial({
      vertexShader: DEFAULT_VERT_SHADER,
      fragmentShader: DEFAULT_FRAG_SHADER,
      transparent: true,
      blending: THREE.AdditiveBlending,
      depthWrite: false,
      uniforms: {
        uTime: { value: 0 },
        uPixelRatio: { value: renderer.getPixelRatio() },
        uScale: { value: 1.0 },
      },
    });

    const points = new THREE.Points(geometry, material);
    scene.add(points);
    pointsRef.current = points;

    // Background starfield
    const starCount = QUALITY_CONFIGS[quality].starCount;
    const starGeo = new THREE.BufferGeometry();
    const starPos = new Float32Array(starCount * 3);
    for (let i = 0; i < starCount; i++) {
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(Math.random() * 2 - 1);
      const dist = 120 + Math.random() * 80;
      starPos[i * 3] = dist * Math.sin(phi) * Math.cos(theta);
      starPos[i * 3 + 1] = dist * Math.cos(phi);
      starPos[i * 3 + 2] = dist * Math.sin(phi) * Math.sin(theta);
    }
    starGeo.setAttribute('position', new THREE.BufferAttribute(starPos, 3));
    const starMat = new THREE.PointsMaterial({
      color: 0x94a3b8,
      size: 1.2,
      transparent: true,
      opacity: 0.65,
    });
    const starField = new THREE.Points(starGeo, starMat);
    scene.add(starField);
    starsRef.current = starField;

    // Interactive Celestial Nodes
    nodeMeshesRef.current = [];
    nodes.forEach((node) => {
      const nodeGeo = new THREE.SphereGeometry(node.radius, 16, 16);
      const nodeMat = new THREE.MeshBasicMaterial({
        color: new THREE.Color(node.color),
        wireframe: true,
      });
      const mesh = new THREE.Mesh(nodeGeo, nodeMat);
      mesh.position.set(...node.position);
      scene.add(mesh);
      nodeMeshesRef.current.push({ mesh, node });
    });

    setIsLoading(false);

    // Animation Loop
    let clock = new THREE.Clock();
    let frameCount = 0;
    let lastFpsUpdate = performance.now();

    const animate = () => {
      animFrameIdRef.current = requestAnimationFrame(animate);
      const elapsedTime = clock.getElapsedTime();

      if (material.uniforms) {
        material.uniforms.uTime.value = elapsedTime;
      }

      // Rotate nodes slightly for dynamic interaction
      nodeMeshesRef.current.forEach(({ mesh }) => {
        mesh.rotation.y += 0.01;
      });

      renderer.render(scene, camera);

      // FPS tracking
      frameCount++;
      const now = performance.now();
      if (now - lastFpsUpdate >= 1000) {
        setFps(Math.round((frameCount * 1000) / (now - lastFpsUpdate)));
        frameCount = 0;
        lastFpsUpdate = now;
      }
    };

    animate();

    // Window Resize Handler
    const handleResize = () => {
      if (!container || !renderer || !camera) return;
      const newW = container.clientWidth;
      const newH = container.clientHeight;
      camera.aspect = newW / newH;
      camera.updateProjectionMatrix();
      renderer.setSize(newW, newH);
      material.uniforms.uPixelRatio.value = renderer.getPixelRatio();
    };

    window.addEventListener('resize', handleResize);

    return () => {
      window.removeEventListener('resize', handleResize);
      if (animFrameIdRef.current) {
        cancelAnimationFrame(animFrameIdRef.current);
      }
      renderer.dispose();
      geometry.dispose();
      material.dispose();
      starGeo.dispose();
      starMat.dispose();
      if (container.contains(renderer.domElement)) {
        container.removeChild(renderer.domElement);
      }
    };
  }, [quality, selectedPaletteKey, updateCameraPosition, nodes]);

  // 2D Canvas Fallback loop
  useEffect(() => {
    if (!is2DFallback) return;
    const canvas = canvas2DRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    let animId: number;
    let startTime = performance.now();

    const render = () => {
      animId = requestAnimationFrame(render);
      const elapsed = performance.now() - startTime;
      const palette = NEBULA_PALETTES[selectedPaletteKey] || NEBULA_PALETTES.orion;
      render2DCanvasFallback(ctx, canvas.width, canvas.height, elapsed, palette);
    };

    render();

    return () => {
      cancelAnimationFrame(animId);
    };
  }, [is2DFallback, selectedPaletteKey]);

  // Mouse & Touch Interaction for Orbiting & Raycasting
  const handleMouseDown = (e: React.MouseEvent) => {
    isDragging.current = true;
    previousMousePosition.current = { x: e.clientX, y: e.clientY };
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    if (!isDragging.current || is2DFallback) return;
    const deltaX = e.clientX - previousMousePosition.current.x;
    const deltaY = e.clientY - previousMousePosition.current.y;

    cameraSpherical.current.theta -= deltaX * 0.008;
    cameraSpherical.current.phi = Math.max(
      0.1,
      Math.min(Math.PI - 0.1, cameraSpherical.current.phi - deltaY * 0.008)
    );

    updateCameraPosition();
    previousMousePosition.current = { x: e.clientX, y: e.clientY };
  };

  const handleMouseUp = () => {
    isDragging.current = false;
  };

  const handleWheel = (e: React.WheelEvent) => {
    if (is2DFallback) return;
    e.preventDefault();
    cameraSpherical.current.radius = Math.max(
      15,
      Math.min(100, cameraSpherical.current.radius + e.deltaY * 0.05)
    );
    updateCameraPosition();
  };

  const handleClick = (e: React.MouseEvent) => {
    if (is2DFallback || !containerRef.current || !cameraRef.current) return;
    const rect = containerRef.current.getBoundingClientRect();
    const mouseX = ((e.clientX - rect.left) / rect.width) * 2 - 1;
    const mouseY = -((e.clientY - rect.top) / rect.height) * 2 + 1;

    const raycaster = new THREE.Raycaster();
    raycaster.setFromCamera(new THREE.Vector2(mouseX, mouseY), cameraRef.current);

    const meshes = nodeMeshesRef.current.map((item) => item.mesh);
    const intersects = raycaster.intersectObjects(meshes);

    if (intersects.length > 0) {
      const hitMesh = intersects[0].object;
      const found = nodeMeshesRef.current.find((item) => item.mesh === hitMesh);
      if (found) {
        setSelectedNode(found.node);
        onNodeSelect?.(found.node);
      }
    } else {
      setSelectedNode(null);
      onNodeSelect?.(null);
    }
  };

  return (
    <div
      style={{
        position: 'relative',
        width,
        height,
        backgroundColor: '#050711',
        overflow: 'hidden',
        userSelect: 'none',
      }}
      className={className}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onWheel={handleWheel}
      onClick={handleClick}
    >
      {/* 3D WebGL Container */}
      {!is2DFallback && (
        <div ref={containerRef} style={{ width: '100%', height: '100%' }} />
      )}

      {/* 2D Canvas Fallback */}
      {is2DFallback && (
        <canvas
          ref={canvas2DRef}
          width={800}
          height={600}
          style={{ width: '100%', height: '100%', display: 'block' }}
        />
      )}

      {/* Loading Overlay */}
      {isLoading && (
        <div
          style={{
            position: 'absolute',
            inset: 0,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            backgroundColor: '#050711',
            color: '#38bdf8',
            fontSize: '16px',
            fontWeight: 600,
          }}
        >
          Initializing 3D Nebula Shaders...
        </div>
      )}

      {/* HUD Header & Controls */}
      <div
        style={{
          position: 'absolute',
          top: 16,
          left: 16,
          right: 16,
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'flex-start',
          pointerEvents: 'none',
        }}
      >
        <div
          style={{
            backgroundColor: 'rgba(15, 23, 42, 0.85)',
            backdropFilter: 'blur(8px)',
            padding: '10px 16px',
            borderRadius: 8,
            border: '1px solid rgba(56, 189, 248, 0.2)',
            color: '#ffffff',
            pointerEvents: 'auto',
          }}
        >
          <div style={{ fontSize: 14, fontWeight: 'bold', color: '#38bdf8' }}>
            Stellar Nebula Exploration
          </div>
          <div style={{ fontSize: 12, color: '#94a3b8', marginTop: 2 }}>
            FPS: {fps} | Particles: {QUALITY_CONFIGS[quality].particleCount.toLocaleString()}
          </div>
        </div>

        {/* Quality and Renderer Controls */}
        <div
          style={{
            display: 'flex',
            gap: 8,
            pointerEvents: 'auto',
            backgroundColor: 'rgba(15, 23, 42, 0.85)',
            backdropFilter: 'blur(8px)',
            padding: '6px 10px',
            borderRadius: 8,
            border: '1px solid rgba(51, 65, 85, 0.8)',
          }}
        >
          {(['low', 'medium', 'high'] as QualityPreset[]).map((q) => (
            <button
              key={q}
              onClick={() => setQuality(q)}
              style={{
                backgroundColor: quality === q ? '#2563eb' : 'transparent',
                color: quality === q ? '#ffffff' : '#94a3b8',
                border: 'none',
                borderRadius: 4,
                padding: '4px 8px',
                fontSize: 12,
                cursor: 'pointer',
                fontWeight: 600,
                textTransform: 'uppercase',
              }}
            >
              {q}
            </button>
          ))}

          {webGLSupported && (
            <button
              onClick={() => setIs2DFallback(!is2DFallback)}
              style={{
                backgroundColor: is2DFallback ? '#d97706' : '#1e293b',
                color: '#ffffff',
                border: '1px solid #475569',
                borderRadius: 4,
                padding: '4px 8px',
                fontSize: 12,
                cursor: 'pointer',
              }}
            >
              {is2DFallback ? '2D Mode' : 'WebGL'}
            </button>
          )}
        </div>
      </div>

      {/* Palette Selector Bar */}
      <div
        style={{
          position: 'absolute',
          bottom: 16,
          left: 16,
          display: 'flex',
          gap: 6,
          backgroundColor: 'rgba(15, 23, 42, 0.85)',
          backdropFilter: 'blur(8px)',
          padding: '6px 12px',
          borderRadius: 8,
          border: '1px solid rgba(51, 65, 85, 0.8)',
        }}
      >
        {Object.entries(NEBULA_PALETTES).map(([key, pal]) => (
          <button
            key={key}
            onClick={() => setSelectedPaletteKey(key)}
            style={{
              backgroundColor: selectedPaletteKey === key ? '#1e3a8a' : 'transparent',
              color: selectedPaletteKey === key ? '#60a5fa' : '#cbd5e1',
              border: selectedPaletteKey === key ? '1px solid #3b82f6' : '1px solid transparent',
              borderRadius: 4,
              padding: '4px 10px',
              fontSize: 12,
              cursor: 'pointer',
              fontWeight: 500,
            }}
          >
            {pal.name}
          </button>
        ))}
      </div>

      {/* Selected Node Details Card */}
      {selectedNode && (
        <div
          style={{
            position: 'absolute',
            bottom: 16,
            right: 16,
            backgroundColor: 'rgba(15, 23, 42, 0.92)',
            backdropFilter: 'blur(10px)',
            padding: 16,
            borderRadius: 10,
            border: `1px solid ${selectedNode.color}`,
            width: 260,
            color: '#ffffff',
          }}
        >
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ fontSize: 14, fontWeight: 'bold', color: selectedNode.color }}>
              {selectedNode.name}
            </span>
            <button
              onClick={() => setSelectedNode(null)}
              style={{
                background: 'none',
                border: 'none',
                color: '#94a3b8',
                cursor: 'pointer',
                fontSize: 14,
              }}
            >
              x
            </button>
          </div>
          <div style={{ fontSize: 12, color: '#94a3b8', marginTop: 4 }}>
            Sector: {selectedNode.details.sector}
          </div>
          <div style={{ fontSize: 12, color: '#94a3b8' }}>
            Yield: {selectedNode.details.yield}
          </div>
          <div style={{ fontSize: 12, color: '#94a3b8' }}>
            Faction: {selectedNode.details.faction}
          </div>
        </div>
      )}
    </div>
  );
};

export default NebulaRenderer;

export type QualityPreset = 'low' | 'medium' | 'high';

export interface QualityConfig {
  particleCount: number;
  starCount: number;
  dprLimit: number;
  enablePostProcessing: boolean;
  fogDensity: number;
  lodDistanceThreshold: number;
}

export const QUALITY_CONFIGS: Record<QualityPreset, QualityConfig> = {
  low: {
    particleCount: 6000,
    starCount: 500,
    dprLimit: 1,
    enablePostProcessing: false,
    fogDensity: 0.015,
    lodDistanceThreshold: 80,
  },
  medium: {
    particleCount: 22000,
    starCount: 1500,
    dprLimit: 1.5,
    enablePostProcessing: true,
    fogDensity: 0.01,
    lodDistanceThreshold: 120,
  },
  high: {
    particleCount: 65000,
    starCount: 4000,
    dprLimit: 2,
    enablePostProcessing: true,
    fogDensity: 0.008,
    lodDistanceThreshold: 160,
  },
};

export interface NebulaPalette {
  name: string;
  coreColor: [number, number, number];
  midColor: [number, number, number];
  outerColor: [number, number, number];
}

export const NEBULA_PALETTES: Record<string, NebulaPalette> = {
  orion: {
    name: 'Orion Veil',
    coreColor: [0.35, 0.75, 1.0],
    midColor: [0.65, 0.25, 0.95],
    outerColor: [0.15, 0.05, 0.45],
  },
  carina: {
    name: 'Carina Flare',
    coreColor: [1.0, 0.85, 0.4],
    midColor: [0.95, 0.25, 0.35],
    outerColor: [0.45, 0.05, 0.25],
  },
  eagle: {
    name: 'Eagle Pillars',
    coreColor: [0.4, 0.95, 0.7],
    midColor: [0.2, 0.6, 0.8],
    outerColor: [0.08, 0.2, 0.35],
  },
  tarantula: {
    name: 'Tarantula Web',
    coreColor: [0.95, 0.95, 1.0],
    midColor: [0.3, 0.45, 0.95],
    outerColor: [0.8, 0.1, 0.4],
  },
};

export interface NebulaParticleData {
  positions: Float32Array;
  colors: Float32Array;
  sizes: Float32Array;
  alphas: Float32Array;
  count: number;
}

export interface CelestialNode {
  id: string;
  name: string;
  type: 'outpost' | 'star' | 'anomaly' | 'mining_claim';
  position: [number, number, number];
  radius: number;
  color: string;
  details: {
    sector: string;
    yield: string;
    faction: string;
  };
}

export function isWebGLAvailable(): boolean {
  try {
    const canvas = document.createElement('canvas');
    return !!(
      window.WebGLRenderingContext &&
      (canvas.getContext('webgl') || canvas.getContext('experimental-webgl'))
    );
  } catch {
    return false;
  }
}

export function isWebGL2Available(): boolean {
  try {
    const canvas = document.createElement('canvas');
    return !!(window.WebGL2RenderingContext && canvas.getContext('webgl2'));
  } catch {
    return false;
  }
}

export function generateNebulaParticles(
  count: number,
  palette: NebulaPalette = NEBULA_PALETTES.orion,
  radius: number = 35
): NebulaParticleData {
  const positions = new Float32Array(count * 3);
  const colors = new Float32Array(count * 3);
  const sizes = new Float32Array(count);
  const alphas = new Float32Array(count);

  for (let i = 0; i < count; i++) {
    // Spiral and radial dispersion
    const spiralAngle = i * 0.02 + (Math.random() - 0.5) * 1.5;
    const distFactor = Math.pow(Math.random(), 1.6);
    const r = distFactor * radius;

    const x = Math.cos(spiralAngle) * r + (Math.random() - 0.5) * 6;
    const y = (Math.random() - 0.5) * (radius * 0.45) * (1 - distFactor * 0.5);
    const z = Math.sin(spiralAngle) * r + (Math.random() - 0.5) * 6;

    positions[i * 3] = x;
    positions[i * 3 + 1] = y;
    positions[i * 3 + 2] = z;

    // Color gradient based on radial distance
    const normDist = Math.min(r / radius, 1.0);
    let rCol: number, gCol: number, bCol: number;

    if (normDist < 0.35) {
      const t = normDist / 0.35;
      rCol = palette.coreColor[0] * (1 - t) + palette.midColor[0] * t;
      gCol = palette.coreColor[1] * (1 - t) + palette.midColor[1] * t;
      bCol = palette.coreColor[2] * (1 - t) + palette.midColor[2] * t;
    } else {
      const t = (normDist - 0.35) / 0.65;
      rCol = palette.midColor[0] * (1 - t) + palette.outerColor[0] * t;
      gCol = palette.midColor[1] * (1 - t) + palette.outerColor[1] * t;
      bCol = palette.midColor[2] * (1 - t) + palette.outerColor[2] * t;
    }

    colors[i * 3] = rCol + (Math.random() - 0.5) * 0.08;
    colors[i * 3 + 1] = gCol + (Math.random() - 0.5) * 0.08;
    colors[i * 3 + 2] = bCol + (Math.random() - 0.5) * 0.08;

    sizes[i] = Math.random() * 22 + 6;
    alphas[i] = (1.0 - normDist * 0.7) * (Math.random() * 0.6 + 0.35);
  }

  return { positions, colors, sizes, alphas, count };
}

export function generateCelestialNodes(): CelestialNode[] {
  return [
    {
      id: 'node-alpha',
      name: 'Alpha Centauri Relay',
      type: 'outpost',
      position: [0, 1.5, 0],
      radius: 1.8,
      color: '#38bdf8',
      details: {
        sector: 'Sector 0-Core',
        yield: 'High Helium-3',
        faction: 'Nomad Consortium',
      },
    },
    {
      id: 'node-pulsar',
      name: 'Vela Pulsar Station',
      type: 'anomaly',
      position: [14, 3, -8],
      radius: 1.5,
      color: '#c084fc',
      details: {
        sector: 'Sector 4-Deep',
        yield: 'Relic Crystals',
        faction: 'Unclaimed',
      },
    },
    {
      id: 'node-forge',
      name: 'Soroban Asteroid Forge',
      type: 'mining_claim',
      position: [-16, -2, 12],
      radius: 1.6,
      color: '#f59e0b',
      details: {
        sector: 'Sector 9-Periphery',
        yield: 'Titanium & Soroban Runes',
        faction: 'Iron Guild',
      },
    },
    {
      id: 'node-beacon',
      name: 'Stellar Beacon 07',
      type: 'star',
      position: [8, -4, 16],
      radius: 2.0,
      color: '#f43f5e',
      details: {
        sector: 'Sector 1-Beacon',
        yield: 'Solar Flux',
        faction: 'Solar Directorate',
      },
    },
  ];
}

export function calculateLODFactor(cameraDistance: number, threshold: number): number {
  if (cameraDistance <= threshold * 0.5) return 1.0;
  if (cameraDistance >= threshold * 1.5) return 0.25;
  const t = (cameraDistance - threshold * 0.5) / threshold;
  return Math.max(0.25, 1.0 - t * 0.75);
}

export function render2DCanvasFallback(
  ctx: CanvasRenderingContext2D,
  width: number,
  height: number,
  time: number,
  palette: NebulaPalette = NEBULA_PALETTES.orion
): void {
  ctx.fillStyle = '#050711';
  ctx.fillRect(0, 0, width, height);

  const cx = width / 2;
  const cy = height / 2;

  // Background stars
  ctx.fillStyle = '#ffffff';
  for (let i = 0; i < 70; i++) {
    const sx = ((i * 137.5) % width);
    const sy = ((i * 269.3) % height);
    const brightness = (Math.sin(time * 0.002 + i) * 0.5 + 0.5) * 0.8 + 0.2;
    ctx.globalAlpha = brightness;
    ctx.fillRect(sx, sy, 1.5, 1.5);
  }

  // Draw layered radial gradients for nebula cloud
  const layers = [
    { radius: Math.min(width, height) * 0.5, col: palette.outerColor, alpha: 0.18, offset: 0 },
    { radius: Math.min(width, height) * 0.35, col: palette.midColor, alpha: 0.28, offset: Math.PI / 4 },
    { radius: Math.min(width, height) * 0.2, col: palette.coreColor, alpha: 0.45, offset: Math.PI / 2 },
  ];

  ctx.globalCompositeOperation = 'screen';

  for (const layer of layers) {
    const angle = time * 0.0003 + layer.offset;
    const ox = cx + Math.cos(angle) * 30;
    const oy = cy + Math.sin(angle) * 20;

    const grad = ctx.createRadialGradient(ox, oy, 0, ox, oy, layer.radius);
    const [r, g, b] = layer.col.map((v) => Math.round(v * 255));
    grad.addColorStop(0, `rgba(${r}, ${g}, ${b}, ${layer.alpha})`);
    grad.addColorStop(0.6, `rgba(${r}, ${g}, ${b}, ${layer.alpha * 0.4})`);
    grad.addColorStop(1, 'rgba(0, 0, 0, 0)');

    ctx.fillStyle = grad;
    ctx.beginPath();
    ctx.arc(ox, oy, layer.radius, 0, Math.PI * 2);
    ctx.fill();
  }

  ctx.globalCompositeOperation = 'source-over';
  ctx.globalAlpha = 1.0;
}

import {
  calculateLODFactor,
  generateNebulaParticles,
  generateCelestialNodes,
  QUALITY_CONFIGS,
  NEBULA_PALETTES,
} from './utils';

describe('Visualization Utilities', () => {
  it('defines valid quality presets', () => {
    expect(QUALITY_CONFIGS.low.particleCount).toBeLessThan(QUALITY_CONFIGS.medium.particleCount);
    expect(QUALITY_CONFIGS.medium.particleCount).toBeLessThan(QUALITY_CONFIGS.high.particleCount);
    expect(QUALITY_CONFIGS.low.dprLimit).toBe(1);
  });

  it('contains celestial nebula color palettes', () => {
    expect(NEBULA_PALETTES.orion).toBeDefined();
    expect(NEBULA_PALETTES.carina).toBeDefined();
    expect(NEBULA_PALETTES.eagle).toBeDefined();
    expect(NEBULA_PALETTES.tarantula).toBeDefined();
    expect(NEBULA_PALETTES.orion.coreColor).toHaveLength(3);
  });

  it('generates particles with valid buffer sizes', () => {
    const count = 100;
    const particles = generateNebulaParticles(count, NEBULA_PALETTES.orion, 20);

    expect(particles.positions.length).toBe(count * 3);
    expect(particles.colors.length).toBe(count * 3);
    expect(particles.sizes.length).toBe(count);
    expect(particles.alphas.length).toBe(count);
    expect(particles.count).toBe(count);
  });

  it('calculates LOD scaling factor according to camera distance', () => {
    const threshold = 100;
    expect(calculateLODFactor(40, threshold)).toBe(1.0);
    expect(calculateLODFactor(160, threshold)).toBe(0.25);

    const midLOD = calculateLODFactor(100, threshold);
    expect(midLOD).toBeGreaterThan(0.25);
    expect(midLOD).toBeLessThanOrEqual(1.0);
  });

  it('generates interactive celestial nodes', () => {
    const nodes = generateCelestialNodes();
    expect(nodes.length).toBeGreaterThanOrEqual(4);
    nodes.forEach((node) => {
      expect(node.id).toBeDefined();
      expect(node.position).toHaveLength(3);
      expect(node.details.sector).toBeDefined();
    });
  });
});

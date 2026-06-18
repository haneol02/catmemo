// breeds.js — cat colour palettes
// Maps original SVG fill values → breed-specific replacements

export const BREEDS = {
  orange: {
    name: '주황 줄무늬', emoji: '🟠',
    colors: {
      '#FEBD92': '#FEBD92', '#431D0B': '#431D0B',
      '#330C02': '#330C02', '#340B01': '#340B01',
      '#3B1000': '#3B1000', '#190502': '#190502',
    },
  },
  black: {
    name: '검은 고양이', emoji: '⬛',
    colors: {
      '#FEBD92': '#3a3a3a', '#431D0B': '#181818',
      '#330C02': '#121212', '#340B01': '#121212',
      '#3B1000': '#161616', '#190502': '#0a0a0a',
    },
  },
  white: {
    name: '흰 고양이', emoji: '⬜',
    colors: {
      '#FEBD92': '#f5f0ea', '#431D0B': '#c8a87a',
      '#330C02': '#b09060', '#340B01': '#b09060',
      '#3B1000': '#b89870', '#190502': '#906848',
    },
  },
  gray: {
    name: '회색 고양이', emoji: '🩶',
    colors: {
      '#FEBD92': '#b0bec5', '#431D0B': '#37474f',
      '#330C02': '#263238', '#340B01': '#263238',
      '#3B1000': '#2d3f48', '#190502': '#1a272e',
    },
  },
  calico: {
    name: '삼색 고양이', emoji: '🐈',
    colors: {
      '#FEBD92': '#f0d4b0', '#431D0B': '#7a3a10',
      '#330C02': '#5c2a0a', '#340B01': '#5c2a0a',
      '#3B1000': '#6a3010', '#190502': '#3c1a06',
    },
  },
};

/** Replace all original SVG fill colours with breed palette */
export function applyBreed(svgText, breedKey) {
  const breed = BREEDS[breedKey] || BREEDS.orange;
  let out = svgText;
  for (const [orig, repl] of Object.entries(breed.colors)) {
    // replace both as-written and uppercase variants
    out = out.split(orig).join(repl);
  }
  return out;
}

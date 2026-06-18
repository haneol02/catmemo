// catcipher.js — Cat Cipher v2
// v2: 4-bit nibble → 1 Korean syllable  (2x shorter than v1)
// v1: 4-bit nibble → 2 Korean syllables (legacy, auto-detected for decode)

// ── v2 charset (16 unique syllables, none overlap with v1) ───────────────────
const CATS = ['냑','냔','냠','뇨','뇩','묘','묵','욱','낙','랑','맹','뭉','봉','꿍','킁','흥'];
const CAT_MAP = Object.fromEntries(CATS.map((c,i)=>[c,i]));

// ── v1 charset (legacy — used only in catDecode fallback) ────────────────────
const V1_ENC = {
  '0000':'냐냐','0001':'냐냥','0010':'냐아','0011':'냐앙',
  '0100':'냐웅','0101':'냐오','0110':'냐옹','0111':'냐야',
  '1000':'냥냐','1001':'냥냥','1010':'냥아','1011':'냥앙',
  '1100':'냥웅','1101':'냥오','1110':'냥옹','1111':'냥야',
};
const V1_DEC = Object.fromEntries(Object.entries(V1_ENC).map(([k,v])=>[v,k]));
const V1_CHARS = new Set([...'냐냥아앙웅오옹야']);

/** Encode text → compact cat sounds (v2: 2 syllables per byte) */
export function catEncode(text) {
  const bytes = new TextEncoder().encode(text);
  return Array.from(bytes, b => CATS[b >> 4] + CATS[b & 0xF]).join('');
}

/** Decode cat sounds → original text (auto-detects v2 or v1 format) */
export function catDecode(catStr) {
  const ch = [...catStr]; // split by Unicode code point
  // Detect format: v2 chars are outside v1 charset
  const isV2 = ch.some(c => !V1_CHARS.has(c));
  if (isV2) {
    // v2: 2 chars per byte
    if (ch.length % 2 !== 0) throw new Error('냐옹...?');
    const bytes = new Uint8Array(ch.length / 2);
    for (let i = 0; i < ch.length; i += 2) {
      const hi = CAT_MAP[ch[i]], lo = CAT_MAP[ch[i+1]];
      if (hi === undefined || lo === undefined) throw new Error('냐옹...?');
      bytes[i / 2] = (hi << 4) | lo;
    }
    return new TextDecoder().decode(bytes);
  } else {
    // v1 legacy: 4 chars per byte
    if (ch.length % 4 !== 0) throw new Error('냐옹...?');
    const bytes = new Uint8Array(ch.length / 4);
    for (let i = 0; i < ch.length; i += 4) {
      const hi = V1_DEC[ch[i]+ch[i+1]], lo = V1_DEC[ch[i+2]+ch[i+3]];
      if (!hi || !lo) throw new Error('냐옹...?');
      bytes[i / 4] = parseInt(hi + lo, 2);
    }
    return new TextDecoder().decode(bytes);
  }
}

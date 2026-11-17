// WebGL2 renderer, for browsers without WebGPU. Same interface and the same
// look as src/renderer.js: one background pass and one instanced draw of
// paper parts, with the shapes ported from src/shader.wgsl to GLSL ES 3.0.
//
// The two shaders must be kept in step: a change to a part's shape, the
// paper shading or the background in src/shader.wgsl belongs here too. The
// part ids are written at load time from the generated PART_IDS, so a new
// part needs only its shape added below.
import { t } from './i18n.js';
import { PART_IDS, INK } from './data.js';
import { loadPartImages } from './renderer.js';

const INSTANCE_FLOATS = 12;
const MAX_INSTANCES = 4096;
const WORLD_HEIGHT = 640; // world units across the shorter screen side, as in renderer.js
const STOP_MOTION_FPS = 8;

const COMMON = /* glsl */ `#version 300 es
precision highp float;
precision highp int;

uniform vec2 uView;     // visible area in world units
uniform vec2 uCam;      // world position at the centre of the screen
uniform vec2 uRes;      // canvas size in device pixels
uniform float uTime;
uniform float uQtime;   // time stepped to the stop-motion frame rate
uniform float uPx;      // world units per device pixel
uniform float uQuality; // 1 for full quality; 0 drops line boil, grain and the drop shadow

float hash21(vec2 p) {
  vec2 q = fract(p * vec2(123.34, 456.21));
  q += dot(q, q + 45.32);
  return fract(q.x * q.y);
}

float vnoise(vec2 p) {
  vec2 i = floor(p);
  vec2 f = fract(p);
  vec2 u = f * f * (3.0 - 2.0 * f);
  return mix(
    mix(hash21(i), hash21(i + vec2(1.0, 0.0)), u.x),
    mix(hash21(i + vec2(0.0, 1.0)), hash21(i + vec2(1.0, 1.0)), u.x),
    u.y);
}

float fbm(vec2 p) {
  float a = 0.5;
  float s = 0.0;
  vec2 q = p;
  for (int i = 0; i < 4; i++) {
    s += a * vnoise(q);
    q = q * 2.03 + vec2(17.1, 9.2);
    a *= 0.5;
  }
  return s;
}
`;

/** The part ids and ink colour as GLSL constants, from the generated tables. */
function constants() {
  const lines = [`const vec3 INK = vec3(${INK.map((v) => v.toFixed(4)).join(', ')});`];
  for (const [name, id] of Object.entries(PART_IDS)) lines.push(`const int PART_${name.toUpperCase()} = ${id};`);
  return lines.join('\n');
}

const PART_VS = /* glsl */ `
layout(location = 0) in vec4 aA;   // x, y, half width (negative mirrors), half height
layout(location = 1) in vec4 aB;   // rotation, part, flash, seed
layout(location = 2) in vec4 aC;   // colour
const float PAD = 10.0;
const vec2 SHADOW_OFF = vec2(4.0, 6.0);
out vec2 vP;
flat out vec2 vHalf;
flat out vec4 vMisc;
flat out vec4 vColor;
flat out vec2 vSoff;
void main() {
  vec2 corners[6] = vec2[6](
    vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(-1.0, 1.0),
    vec2(-1.0, 1.0), vec2(1.0, -1.0), vec2(1.0, 1.0));
  vec2 half_ = abs(aA.zw);
  float flip = aA.z < 0.0 ? -1.0 : 1.0;
  vec2 l = corners[gl_VertexID] * (half_ + vec2(PAD));
  float cs = cos(aB.x);
  float sn = sin(aB.x);
  vec2 w = aA.xy + vec2(l.x * cs - l.y * sn, l.x * sn + l.y * cs);
  vec2 ndc = (w - uCam) / (uView * 0.5);
  gl_Position = vec4(ndc.x, -ndc.y, 0.0, 1.0);
  vP = vec2(l.x * flip, l.y);
  vHalf = half_;
  vMisc = aB;
  vColor = aC;
  vec2 so = vec2(SHADOW_OFF.x * cs + SHADOW_OFF.y * sn, -SHADOW_OFF.x * sn + SHADOW_OFF.y * cs);
  vSoff = vec2(so.x * flip, so.y);
}
`;

const PART_FS = /* glsl */ `
uniform vec4 uRects[32];   // atlas rect per part id (x, y, w, h); w = 0 means no image
uniform sampler2D uAtlas;
in vec2 vP;
flat in vec2 vHalf;
flat in vec4 vMisc;
flat in vec4 vColor;
flat in vec2 vSoff;
out vec4 outColor;

const vec3 PAPER = vec3(0.965, 0.94, 0.875);
const float MARGIN = 3.5;

float sdSeg(vec2 p, vec2 a, vec2 b) {
  vec2 pa = p - a;
  vec2 ba = b - a;
  float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
  return length(pa - ba * h);
}

float sdBox(vec2 p, vec2 b) {
  vec2 d = abs(p) - b;
  return length(max(d, vec2(0.0))) + min(max(d.x, d.y), 0.0);
}

float dot2(vec2 v) { return dot(v, v); }

float sdHeart(vec2 p0) {
  vec2 p = vec2(abs(p0.x), p0.y);
  if (p.y + p.x > 1.0) return sqrt(dot2(p - vec2(0.25, 0.75))) - 0.35355;
  return sqrt(min(dot2(p - vec2(0.0, 1.0)), dot2(p - 0.5 * max(p.x + p.y, 0.0)))) * sign(p.x - p.y);
}

struct Shape { float d; float ink; float light; };

float dab(float d) { return smoothstep(0.03, -0.03, d); }

// p is in the part's own space (x right, y down), h is its half size.
Shape shape(int part, vec2 p, vec2 h, float seed) {
  vec2 q = p / h;
  float m = min(h.x, h.y);
  Shape s = Shape(1e3, 0.0, 0.0);
  if (part == PART_PLAYER_BODY) {
    float t = q.y * 0.5 + 0.5;
    float w = mix(0.42, 0.95, t * (0.4 + 0.6 * t));
    float hem = 1.0 - 0.14 * abs(fract(q.x * 2.0 + 0.5) * 2.0 - 1.0);
    s.d = max(abs(q.x) - w, max(q.y - hem, -q.y - 1.0)) * m;
    s.ink = 0.55 * dab(abs(q.y + 0.2) - 0.07);
  } else if (part == PART_PLAYER_HEAD) {
    s.d = (length(q) - 0.95) * m;
    float face = dab(length((q - vec2(0.18, 0.12)) / vec2(0.62, 0.6)) - 1.0);
    float eyes = dab(min(length(q - vec2(0.02, 0.1)), length(q - vec2(0.46, 0.1))) - 0.12);
    s.ink = 0.9 * face * (1.0 - eyes);
    s.light = eyes;
  } else if (part == PART_SCYTHE) {
    float handle = sdSeg(q, vec2(-0.35, 1.0), vec2(-0.35, -0.85)) - 0.08;
    float a = length(q - vec2(0.0, -0.35)) - 0.62;
    float b = length(q - vec2(0.05, -0.1)) - 0.68;
    float blade = max(max(a, -b), -0.45 - q.x);
    s.d = min(handle, blade) * m;
    s.light = 0.5 * dab(blade);
  } else if (part == PART_DAO) {
    float grip = sdSeg(q, vec2(0.0, 1.0), vec2(0.0, 0.6)) - 0.09;
    float guard = sdBox(q - vec2(0.0, 0.52), vec2(0.3, 0.05));
    float t = clamp((0.45 - q.y) / 1.4, 0.0, 1.0);
    float spine = 0.32 * t * t;
    float blade = max(abs(q.x - spine - 0.05) - (0.2 + 0.08 * t) * (1.0 - t * t * t * 0.9), max(q.y - 0.47, -0.97 - q.y));
    s.d = min(min(grip, guard), blade) * m;
    s.light = 0.5 * dab(blade + 0.04);
  } else if (part == PART_DADAO) {
    float pole = sdSeg(q, vec2(-0.15, 1.0), vec2(-0.15, -0.3)) - 0.07;
    float e = length((q - vec2(0.1, -0.5)) / vec2(0.42, 0.48)) - 1.0;
    float blade = max(e * 0.42, -0.2 - q.x);
    s.d = min(pole, blade) * m;
    s.light = 0.5 * dab(blade + 0.03);
  } else if (part == PART_BLOT) {
    float ang = atan(q.y, q.x);
    float r = 0.82 + 0.07 * sin(ang * 5.0 + seed * 40.0) + 0.05 * sin(ang * 3.0 - uQtime * 2.0);
    s.d = (length(q) - r) * m;
    s.light = dab(min(length(q - vec2(-0.2, -0.12)), length(q - vec2(0.38, -0.12))) - 0.2);
    s.ink = dab(min(length(q - vec2(-0.13, -0.1)), length(q - vec2(0.45, -0.1))) - 0.09);
  } else if (part == PART_WISP) {
    float body = (length(q / vec2(0.42, 0.75)) - 1.0) * 0.42;
    float ax = abs(q.x);
    float wing = max(abs(q.y + 0.15) - (1.0 - ax) * 0.6, ax - 0.98);
    s.d = min(body, wing) * m;
    s.light = dab(min(length(q - vec2(-0.14, -0.15)), length(q - vec2(0.18, -0.15))) - 0.1);
  } else if (part == PART_BRUTE) {
    float body = sdBox(q - vec2(0.0, 0.15), vec2(0.5, 0.45)) - 0.3;
    vec2 a = vec2(abs(q.x), q.y);
    float horn = sdSeg(a, vec2(0.4, -0.5), vec2(0.72, -0.93)) - 0.1;
    s.d = min(body, horn) * m;
    s.light = dab(sdSeg(a, vec2(0.14, -0.02), vec2(0.44, -0.16)) - 0.075);
    s.ink = dab(sdSeg(q, vec2(-0.3, 0.5), vec2(0.3, 0.5)) - 0.045);
  } else if (part == PART_GEM) {
    s.d = (abs(q.x) * 1.2 + abs(q.y) - 0.95) * 0.64 * m;
    s.light = 0.45 * smoothstep(0.05, -0.05, q.x + q.y + 0.1);
  } else if (part == PART_SLASH) {
    float ang = atan(q.y, q.x);
    float t = clamp(1.0 - abs(ang) / 1.6, 0.0, 1.0);
    s.d = (abs(length(q) - 0.78) - 0.19 * t) * m;
    if (t <= 0.0) s.d = 1e3;
  } else if (part == PART_TALISMAN) {
    s.d = sdBox(q, vec2(0.88, 0.92)) * m;
    s.ink = 0.75 * step(abs(q.x), 0.5) * step(fract(q.y * 2.2 + 0.2), 0.32);
  } else if (part == PART_BEAD) {
    s.d = (length(q) - 0.9) * m;
    s.light = 0.7 * smoothstep(0.32, 0.2, length(q - vec2(-0.3, -0.3)));
    s.ink = dab(length(q) - 0.13);
  } else if (part == PART_SPLAT) {
    float ang = atan(q.y, q.x);
    float r = 0.5 + 0.45 * vnoise(vec2(ang * 1.6 + 5.0, seed * 50.0));
    s.d = (length(q) - r) * m;
  } else if (part == PART_GRASS) {
    float a = sdSeg(q, vec2(-0.45, 0.9), vec2(-0.75, -0.2)) - 0.08;
    float b = sdSeg(q, vec2(0.0, 0.9), vec2(0.06, -0.8)) - 0.09;
    float c = sdSeg(q, vec2(0.45, 0.9), vec2(0.8, -0.35)) - 0.08;
    s.d = min(a, min(b, c)) * m;
  } else if (part == PART_STONE) {
    s.d = (length(q / vec2(0.95, 0.7)) - 1.0) * 0.7 * m;
  } else if (part == PART_HEART) {
    s.d = sdHeart(vec2(q.x * 0.62, (1.0 - q.y) * 0.52)) * m * 1.6;
    s.light = 0.4 * smoothstep(0.2, 0.1, length(q - vec2(-0.4, -0.35)));
  } else if (part == PART_RING) {
    s.d = (abs(length(q) - 0.9) - 0.05) * m;
  } else if (part == PART_SCRAP) {
    s.d = sdBox(q, vec2(0.9, 0.9)) * m;
  } else if (part == PART_DIGIT) {
    int n = clamp(int(seed * 10.0 + 0.5), 0, 9);
    int masks[10] = int[10](0x3f, 0x06, 0x5b, 0x4f, 0x66, 0x6d, 0x7d, 0x07, 0x7f, 0x6f);
    int mask = masks[n];
    float w = 0.62;
    vec2 k = vec2(q.x + q.y * 0.12, q.y);
    float d = 1e3;
    if ((mask & 0x01) != 0) d = min(d, sdSeg(k, vec2(-w, -0.9), vec2(w, -0.9)));
    if ((mask & 0x02) != 0) d = min(d, sdSeg(k, vec2(w, -0.9), vec2(w, 0.0)));
    if ((mask & 0x04) != 0) d = min(d, sdSeg(k, vec2(w, 0.0), vec2(w, 0.9)));
    if ((mask & 0x08) != 0) d = min(d, sdSeg(k, vec2(-w, 0.9), vec2(w, 0.9)));
    if ((mask & 0x10) != 0) d = min(d, sdSeg(k, vec2(-w, 0.0), vec2(-w, 0.9)));
    if ((mask & 0x20) != 0) d = min(d, sdSeg(k, vec2(-w, -0.9), vec2(-w, 0.0)));
    if ((mask & 0x40) != 0) d = min(d, sdSeg(k, vec2(-w, 0.0), vec2(w, 0.0)));
    s.d = (d - 0.2) * m;
  } else if (part == PART_GROUND_SHADOW) {
    s.d = (length(q) - 0.6) * m;
  } else {
    s.d = (length(q) - 0.95) * m;
  }
  return s;
}

bool isFlat(int part) {
  return part == PART_SLASH || part == PART_SPLAT || part == PART_GRASS || part == PART_STONE
    || part == PART_RING || part == PART_SCRAP || part == PART_GROUND_SHADOW;
}

vec4 img(vec4 rect, vec2 q) {
  float inside = step(0.0, q.x) * step(q.x, 1.0) * step(0.0, q.y) * step(q.y, 1.0);
  return textureLod(uAtlas, rect.xy + clamp(q, vec2(0.0), vec2(1.0)) * rect.zw, 0.0) * inside;
}

void main() {
  int part = int(vMisc.y + 0.5);
  float seed = vMisc.w;
  float flash = vMisc.z;
  vec2 h = vHalf;
  float aa = uPx;
  bool flatPart = isFlat(part);
  bool full = uQuality > 0.5;

  vec2 bo = vec2(seed * 17.0 + uQtime * 3.1, seed * 5.0 - uQtime * 1.7);
  vec2 boil = vec2(vnoise(vP * 0.09 + bo), vnoise(vP * 0.09 + bo + 31.7)) - 0.5;
  float amp = (flatPart ? 1.0 : 2.0) * (full ? 1.0 : 0.0);
  vec2 p = vP + boil * amp;
  float grain = 0.5;
  if (full) grain = vnoise(vP * 1.3 + seed * 9.0) * 0.6 + vnoise(vP * 0.35 + seed * 3.0) * 0.4;

  float fill = 0.0;
  float margin = 0.0;
  float shadow = 0.0;
  vec3 col = vColor.rgb;

  vec4 rect = uRects[clamp(part, 0, 31)];
  if (rect.z > 0.0) {
    vec2 q = p / h * 0.5 + 0.5;
    vec4 tx = img(rect, q);
    fill = tx.a;
    col = tx.rgb;
    if (!flatPart) {
      vec2 r = MARGIN / (2.0 * h);
      float mx = tx.a;
      for (int k = 0; k < 8; k++) {
        float ang = float(k) * 0.7853982;
        mx = max(mx, img(rect, q + vec2(cos(ang), sin(ang)) * r).a);
      }
      margin = mx;
      shadow = img(rect, q - vSoff / (2.0 * h)).a * 0.3;
    }
  } else {
    Shape s = shape(part, p, h, seed);
    if (part == PART_GROUND_SHADOW) {
      fill = smoothstep(0.4 * min(h.x, h.y), -0.3 * min(h.x, h.y), s.d);
    } else {
      fill = smoothstep(aa, -aa, s.d);
    }
    if (!flatPart) {
      margin = smoothstep(aa, -aa, s.d - MARGIN);
      if (full) {
        float sd = shape(part, p - vSoff, h, seed).d;
        shadow = smoothstep(3.0, -3.0, sd - MARGIN) * 0.3;
      }
      col = col * (0.9 + 0.2 * grain);
      col = mix(col, INK, smoothstep(-3.2, -1.2, s.d) * 0.7);
      col = mix(col, PAPER, s.light);
      col = mix(col, INK, s.ink);
    } else {
      col = col * (0.92 + 0.16 * grain);
    }
  }

  col = mix(col, vec3(1.0), flash * 0.85);

  if (flatPart) {
    float a = fill * vColor.a * (0.8 + 0.2 * grain);
    if (a < 0.004) discard;
    outColor = vec4(col, a);
    return;
  }

  vec3 paper = PAPER * (0.95 + 0.07 * grain);
  vec3 rgb = mix(paper, col, fill);
  float under = shadow * (1.0 - margin);
  float a = margin + under;
  if (a < 0.004) discard;
  vec3 outc = (rgb * margin + INK * under) / a;
  outColor = vec4(outc, a * vColor.a);
}
`;

const BG_VS = /* glsl */ `
void main() {
  vec2 tri[3] = vec2[3](vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
  gl_Position = vec4(tri[gl_VertexID], 0.0, 1.0);
}
`;

const BG_FS = /* glsl */ `
out vec4 outColor;
void main() {
  // gl_FragCoord starts at the bottom; the WebGPU version's starts at the top.
  vec2 uv = vec2(gl_FragCoord.x, uRes.y - gl_FragCoord.y) / uRes;
  vec2 w = uCam + (uv - 0.5) * uView;
  vec3 col = vec3(0.925, 0.885, 0.79);
  col = mix(col, vec3(0.86, 0.79, 0.66), smoothstep(0.35, 0.8, fbm(w * 0.0035)));
  col = mix(col, vec3(0.62, 0.71, 0.64), 0.35 * smoothstep(0.55, 0.75, fbm(w * 0.0016 + vec2(7.3, 2.1))));
  col *= 0.97 + 0.04 * vnoise(w * vec2(0.9, 0.12)) + 0.03 * vnoise(w * 1.7);
  float v = length(uv - 0.5) * 1.3;
  col *= 1.0 - 0.3 * v * v * v;
  outColor = vec4(col, 1.0);
}
`;

function compile(gl, vsText, fsText) {
  const make = (type, text) => {
    const s = gl.createShader(type);
    gl.shaderSource(s, text);
    gl.compileShader(s);
    if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) throw new Error(t('error.shader', { detail: gl.getShaderInfoLog(s) }));
    return s;
  };
  const program = gl.createProgram();
  gl.attachShader(program, make(gl.VERTEX_SHADER, vsText));
  gl.attachShader(program, make(gl.FRAGMENT_SHADER, fsText));
  gl.linkProgram(program);
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) throw new Error(t('error.shader', { detail: gl.getProgramInfoLog(program) }));
  return program;
}

export async function createRendererGL(canvas) {
  const gl = canvas.getContext('webgl2', { alpha: false, antialias: false, premultipliedAlpha: false });
  if (!gl) throw new Error(t('error.noWebgpu'));
  const head = `${COMMON}\n${constants()}\n`;
  const partProgram = compile(gl, head + PART_VS, head + PART_FS);
  const bgProgram = compile(gl, head + BG_VS, head + BG_FS);
  const parts = await loadPartImages();

  const instBuf = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, instBuf);
  gl.bufferData(gl.ARRAY_BUFFER, MAX_INSTANCES * INSTANCE_FLOATS * 4, gl.DYNAMIC_DRAW);
  const vao = gl.createVertexArray();
  gl.bindVertexArray(vao);
  for (let k = 0; k < 3; k++) {
    gl.enableVertexAttribArray(k);
    gl.vertexAttribPointer(k, 4, gl.FLOAT, false, INSTANCE_FLOATS * 4, k * 16);
    gl.vertexAttribDivisor(k, 1);
  }
  gl.bindVertexArray(null);
  const emptyVao = gl.createVertexArray();

  const atlas = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, atlas);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  if (parts.canvas) {
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, parts.canvas);
  } else {
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array(4));
  }

  const uniforms = (program) => {
    const u = {};
    for (const name of ['uView', 'uCam', 'uRes', 'uTime', 'uQtime', 'uPx', 'uQuality', 'uRects', 'uAtlas']) {
      u[name] = gl.getUniformLocation(program, name);
    }
    return u;
  };
  const partU = uniforms(partProgram);
  const bgU = uniforms(bgProgram);
  gl.useProgram(partProgram);
  gl.uniform4fv(partU.uRects, parts.rects);
  gl.uniform1i(partU.uAtlas, 0);

  const view = { w: 1280, h: 720, px: 1 };
  let quality = 1;

  /** Match the canvas to its CSS size. Returns the visible area in world units. */
  function resize() {
    const small = Math.min(window.innerWidth, window.innerHeight) < 600;
    const cap = quality ? (small ? 1.5 : 2) : 1;
    const dpr = Math.min(window.devicePixelRatio || 1, cap);
    const w = Math.max(1, Math.round(canvas.clientWidth * dpr));
    const h = Math.max(1, Math.round(canvas.clientHeight * dpr));
    if (canvas.width !== w || canvas.height !== h) {
      canvas.width = w;
      canvas.height = h;
    }
    view.px = WORLD_HEIGHT / Math.min(w, h);
    view.w = w * view.px;
    view.h = h * view.px;
    return view;
  }

  function setCommon(u, camX, camY, time) {
    gl.uniform2f(u.uView, view.w, view.h);
    gl.uniform2f(u.uCam, camX, camY);
    gl.uniform2f(u.uRes, canvas.width, canvas.height);
    gl.uniform1f(u.uTime, time);
    gl.uniform1f(u.uQtime, Math.floor(time * STOP_MOTION_FPS));
    gl.uniform1f(u.uPx, view.px);
    gl.uniform1f(u.uQuality, quality);
  }

  /** Draw one frame; the same arguments as the WebGPU renderer's `draw`. */
  function draw(memory, ptr, count, camX, camY, time) {
    gl.viewport(0, 0, canvas.width, canvas.height);
    gl.disable(gl.BLEND);
    gl.useProgram(bgProgram);
    setCommon(bgU, camX, camY, time);
    gl.bindVertexArray(emptyVao);
    gl.drawArrays(gl.TRIANGLES, 0, 3);

    const n = Math.min(count, MAX_INSTANCES);
    if (n > 0) {
      gl.bindBuffer(gl.ARRAY_BUFFER, instBuf);
      gl.bufferSubData(gl.ARRAY_BUFFER, 0, new Float32Array(memory, ptr, n * INSTANCE_FLOATS));
      gl.enable(gl.BLEND);
      gl.blendFuncSeparate(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA, gl.ONE, gl.ONE_MINUS_SRC_ALPHA);
      gl.useProgram(partProgram);
      setCommon(partU, camX, camY, time);
      gl.activeTexture(gl.TEXTURE0);
      gl.bindTexture(gl.TEXTURE_2D, atlas);
      gl.bindVertexArray(vao);
      gl.drawArraysInstanced(gl.TRIANGLES, 0, 6, n);
      gl.bindVertexArray(null);
    }
  }

  function setQuality(full) {
    quality = full ? 1 : 0;
  }

  return { resize, draw, setQuality, kind: 'webgl2' };
}

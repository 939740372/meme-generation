import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const outputDir = path.resolve(__dirname, "../src-tauri/stickers");
fs.mkdirSync(outputDir, { recursive: true });

export const STICKERS_METADATA = [
  {
    id: "deal_with_it",
    name: "暴徒墨镜",
    category: "classic",
    defaultWidth: 150,
    svg: `<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256" fill="none">
      <!-- Deal With It Sunglasses 8-bit Pixelated -->
      <g transform="translate(18, 92)">
        <!-- Bridge -->
        <rect x="88" y="24" width="44" height="12" fill="#050505"/>
        <rect x="92" y="28" width="36" height="4" fill="#ffffff" opacity="0.3"/>
        <!-- Left Lens Outline & Fill -->
        <path d="M 0 0 H 88 V 16 H 80 V 32 H 72 V 48 H 64 V 60 H 12 V 48 H 4 V 32 H 0 Z" fill="#080808"/>
        <!-- Right Lens Outline & Fill -->
        <path d="M 132 0 H 220 V 16 H 212 V 32 H 204 V 48 H 196 V 60 H 144 V 48 H 136 V 32 H 132 Z" fill="#080808"/>
        <!-- Specular Highlights White Pixels (Deal with it classic reflection) -->
        <rect x="12" y="8" width="12" height="12" fill="#ffffff"/>
        <rect x="24" y="20" width="12" height="12" fill="#ffffff"/>
        <rect x="36" y="32" width="12" height="12" fill="#ffffff"/>
        <rect x="144" y="8" width="12" height="12" fill="#ffffff"/>
        <rect x="156" y="20" width="12" height="12" fill="#ffffff"/>
        <rect x="168" y="32" width="12" height="12" fill="#ffffff"/>
        <!-- Sub-reflection dots -->
        <rect x="60" y="44" width="8" height="8" fill="#e0e0e0" opacity="0.8"/>
        <rect x="192" y="44" width="8" height="8" fill="#e0e0e0" opacity="0.8"/>
      </g>
    </svg>`,
  },
  {
    id: "red_question",
    name: "经典问号",
    category: "classic",
    defaultWidth: 90,
    svg: `<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256" fill="none">
      <defs>
        <filter id="qShadow" x="-10%" y="-10%" width="130%" height="130%">
          <feDropShadow dx="4" dy="7" stdDeviation="5" flood-color="#000000" flood-opacity="0.6"/>
        </filter>
        <linearGradient id="qGrad" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0%" stop-color="#ff4444"/>
          <stop offset="60%" stop-color="#dd0000"/>
          <stop offset="100%" stop-color="#990000"/>
        </linearGradient>
      </defs>
      <g filter="url(#qShadow)">
        <path d="M 128 32 C 90 32 64 58 64 96 C 64 105 72 112 82 112 C 92 112 100 104 100 95 C 100 75 110 62 128 62 C 146 62 158 74 158 90 C 158 106 146 118 132 130 C 114 146 108 162 108 184 C 108 193 115 200 124 200 C 133 200 140 193 140 184 C 140 168 144 158 158 146 C 176 130 190 114 190 88 C 190 54 162 32 128 32 Z" fill="url(#qGrad)" stroke="#ffffff" stroke-width="4"/>
        <circle cx="124" cy="226" r="15" fill="url(#qGrad)" stroke="#ffffff" stroke-width="4"/>
      </g>
    </svg>`,
  },
  {
    id: "confused_questions",
    name: "连环问号",
    category: "classic",
    defaultWidth: 130,
    svg: `<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256" fill="none">
      <defs>
        <filter id="cShadow" x="-20%" y="-20%" width="140%" height="140%">
          <feDropShadow dx="3" dy="5" stdDeviation="4" flood-color="#000000" flood-opacity="0.5"/>
        </filter>
      </defs>
      <!-- Left small question -->
      <g transform="translate(18, 60) rotate(-18 30 50)" filter="url(#cShadow)">
        <path d="M 30 10 C 16 10 6 22 6 36 C 6 42 10 46 16 46 C 22 46 26 42 26 36 C 26 28 30 22 38 22 C 46 22 52 28 52 34 C 52 42 46 48 40 54 C 32 62 30 70 30 80 H 42 C 42 72 44 68 50 62 C 58 54 64 46 64 34 C 64 18 50 10 30 10 Z" fill="#ffd000" stroke="#000000" stroke-width="2.5"/>
        <circle cx="36" cy="95" r="6" fill="#ffd000" stroke="#000000" stroke-width="2.5"/>
      </g>
      <!-- Center big question -->
      <g transform="translate(90, 20)" filter="url(#cShadow)">
        <path d="M 45 10 C 24 10 10 26 10 46 C 10 54 16 60 24 60 C 32 60 38 54 38 46 C 38 36 44 26 56 26 C 68 26 76 34 76 44 C 76 54 68 62 60 70 C 48 80 44 92 44 106 H 60 C 60 96 64 88 72 80 C 84 68 94 58 94 42 C 94 22 74 10 45 10 Z" fill="#ff3b30" stroke="#000000" stroke-width="3.5"/>
        <circle cx="52" cy="126" r="9" fill="#ff3b30" stroke="#000000" stroke-width="3.5"/>
      </g>
      <!-- Right medium question -->
      <g transform="translate(160, 70) rotate(16 35 50)" filter="url(#cShadow)">
        <path d="M 35 10 C 18 10 8 22 8 38 C 8 44 14 48 20 48 C 26 48 30 44 30 38 C 30 30 34 24 42 24 C 50 24 56 30 56 36 C 56 44 50 50 44 56 C 36 64 32 72 32 84 H 46 C 46 76 48 70 54 64 C 62 56 70 48 70 36 C 70 20 54 10 35 10 Z" fill="#ff9500" stroke="#000000" stroke-width="3"/>
        <circle cx="39" cy="100" r="7.5" fill="#ff9500" stroke="#000000" stroke-width="3"/>
      </g>
    </svg>`,
  },
  {
    id: "doge_face",
    name: "旺柴狗头",
    category: "classic",
    defaultWidth: 130,
    svg: `<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256" fill="none">
      <defs>
        <radialGradient id="dogeFur" cx="50%" cy="45%" r="50%">
          <stop offset="0%" stop-color="#f8d578"/>
          <stop offset="70%" stop-color="#e5a93c"/>
          <stop offset="100%" stop-color="#cb8c22"/>
        </radialGradient>
      </defs>
      <!-- Head -->
      <ellipse cx="128" cy="136" rx="96" ry="86" fill="url(#dogeFur)" stroke="#503006" stroke-width="6"/>
      <!-- Ears -->
      <polygon points="52,90 28,26 94,54" fill="#e5a93c" stroke="#503006" stroke-width="5" stroke-linejoin="round"/>
      <polygon points="56,82 40,40 84,58" fill="#e89874"/>
      <polygon points="204,90 228,26 162,54" fill="#e5a93c" stroke="#503006" stroke-width="5" stroke-linejoin="round"/>
      <polygon points="200,82 216,40 172,58" fill="#e89874"/>
      <!-- White Muzzle -->
      <ellipse cx="128" cy="162" rx="58" ry="44" fill="#fffaf0" stroke="#503006" stroke-width="4"/>
      <!-- Nose -->
      <ellipse cx="128" cy="144" rx="14" ry="9" fill="#1e140a"/>
      <!-- Mouth line & Smirk -->
      <path d="M 128 153 V 165 C 128 174 114 178 98 172 M 128 165 C 128 174 142 178 158 172" stroke="#503006" stroke-width="4.5" stroke-linecap="round"/>
      <!-- Side-Eye Eyeballs -->
      <ellipse cx="88" cy="120" rx="19" ry="14" fill="#ffffff" stroke="#503006" stroke-width="4"/>
      <ellipse cx="80" cy="118" rx="8" ry="9" fill="#2d1704"/>
      <circle cx="78" cy="115" r="2.5" fill="#ffffff"/>
      <ellipse cx="168" cy="120" rx="19" ry="14" fill="#ffffff" stroke="#503006" stroke-width="4"/>
      <ellipse cx="160" cy="118" rx="8" ry="9" fill="#2d1704"/>
      <circle cx="158" cy="115" r="2.5" fill="#ffffff"/>
      <!-- Eyebrows / Furrow -->
      <path d="M 72 96 Q 88 88 106 96" stroke="#482b09" stroke-width="5" stroke-linecap="round"/>
      <path d="M 150 96 Q 168 88 184 96" stroke="#482b09" stroke-width="5" stroke-linecap="round"/>
      <!-- Smug Blush -->
      <ellipse cx="64" cy="148" rx="14" ry="8" fill="#ff7f7f" opacity="0.45"/>
      <ellipse cx="192" cy="148" rx="14" ry="8" fill="#ff7f7f" opacity="0.45"/>
    </svg>`,
  },
  {
    id: "panda_face",
    name: "搞笑熊猫头",
    category: "classic",
    defaultWidth: 130,
    svg: `<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256" fill="none">
      <!-- Panda Ears -->
      <circle cx="64" cy="68" r="34" fill="#111111" stroke="#000000" stroke-width="4"/>
      <circle cx="192" cy="68" r="34" fill="#111111" stroke="#000000" stroke-width="4"/>
      <!-- Panda Head Outline -->
      <ellipse cx="128" cy="140" rx="92" ry="84" fill="#ffffff" stroke="#111111" stroke-width="7"/>
      <!-- Classic expressive eyes with bags -->
      <ellipse cx="88" cy="128" rx="20" ry="14" fill="#111111"/>
      <ellipse cx="88" cy="128" rx="15" ry="10" fill="#ffffff"/>
      <circle cx="86" cy="126" r="6" fill="#111111"/>
      <circle cx="84" cy="124" r="2" fill="#ffffff"/>
      <ellipse cx="168" cy="128" rx="20" ry="14" fill="#111111"/>
      <ellipse cx="168" cy="128" rx="15" ry="10" fill="#ffffff"/>
      <circle cx="170" cy="126" r="6" fill="#111111"/>
      <circle cx="168" cy="124" r="2" fill="#ffffff"/>
      <!-- Eyebrows (wavy hilarious meme look) -->
      <path d="M 68 102 C 78 94 92 108 106 100" stroke="#111111" stroke-width="5" stroke-linecap="round"/>
      <path d="M 150 100 C 164 108 178 94 188 102" stroke="#111111" stroke-width="5" stroke-linecap="round"/>
      <!-- Nose -->
      <polygon points="128,146 120,154 136,154" fill="#111111"/>
      <!-- Smirk / Wide Grin -->
      <path d="M 86 172 Q 128 206 170 172" stroke="#111111" stroke-width="6" stroke-linecap="round" fill="none"/>
      <path d="M 82 168 C 86 174 90 176 96 170" stroke="#111111" stroke-width="4" stroke-linecap="round"/>
      <path d="M 174 168 C 170 174 166 176 160 170" stroke="#111111" stroke-width="4" stroke-linecap="round"/>
    </svg>`,
  },
  {
    id: "clown_nose",
    name: "小丑红鼻",
    category: "classic",
    defaultWidth: 100,
    svg: `<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256" fill="none">
      <defs>
        <radialGradient id="clownGrad" cx="35%" cy="30%" r="70%">
          <stop offset="0%" stop-color="#ff7f7f"/>
          <stop offset="45%" stop-color="#ff0000"/>
          <stop offset="100%" stop-color="#8b0000"/>
        </radialGradient>
      </defs>
      <!-- Big red shiny ball -->
      <circle cx="128" cy="120" r="58" fill="url(#clownGrad)" stroke="#500000" stroke-width="5"/>
      <!-- Specular shiny reflection -->
      <ellipse cx="106" cy="98" rx="16" ry="10" fill="#ffffff" opacity="0.85" transform="rotate(-30 106 98)"/>
      <circle cx="138" cy="146" r="6" fill="#ff9999" opacity="0.6"/>
      <!-- Clown cute smile underneath -->
      <path d="M 82 192 Q 128 226 174 192" stroke="#d00000" stroke-width="7" stroke-linecap="round" fill="none"/>
      <circle cx="78" cy="192" r="5" fill="#d00000"/>
      <circle cx="178" cy="192" r="5" fill="#d00000"/>
    </svg>`,
  },
  {
    id: "sweat_smile",
    name: "流汗黄豆",
    category: "emotion",
    defaultWidth: 120,
    svg: `<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256" fill="none">
      <defs>
        <radialGradient id="faceGrad" cx="45%" cy="40%" r="60%">
          <stop offset="0%" stop-color="#ffea6c"/>
          <stop offset="80%" stop-color="#fdb813"/>
          <stop offset="100%" stop-color="#d99200"/>
        </radialGradient>
        <linearGradient id="sweatGrad" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0%" stop-color="#80d4ff"/>
          <stop offset="100%" stop-color="#0088cc"/>
        </linearGradient>
      </defs>
      <!-- Yellow face -->
      <circle cx="128" cy="136" r="92" fill="url(#faceGrad)" stroke="#a66e00" stroke-width="6"/>
      <!-- Smiling crescent eyes -->
      <path d="M 68 124 Q 92 100 116 124" stroke="#4a3000" stroke-width="7" stroke-linecap="round" fill="none"/>
      <path d="M 140 124 Q 164 100 188 124" stroke="#4a3000" stroke-width="7" stroke-linecap="round" fill="none"/>
      <!-- Strained Awkward Open Grin -->
      <path d="M 78 152 Q 128 178 178 152 Q 168 204 128 204 Q 88 204 78 152 Z" fill="#ffffff" stroke="#4a3000" stroke-width="6" stroke-linejoin="round"/>
      <!-- Teeth divider line -->
      <path d="M 84 166 Q 128 184 172 166" stroke="#c0a060" stroke-width="3"/>
      <!-- Huge sweat drop on forehead -->
      <path d="M 198 94 C 198 120 180 138 168 138 C 156 138 146 120 146 104 C 146 84 172 44 172 44 C 172 44 198 74 198 94 Z" fill="url(#sweatGrad)" stroke="#006699" stroke-width="4"/>
      <ellipse cx="162" cy="100" rx="4" ry="10" fill="#ffffff" opacity="0.8" transform="rotate(-20 162 100)"/>
    </svg>`,
  },
  {
    id: "approved_stamp",
    name: "绝赞印章",
    category: "emotion",
    defaultWidth: 140,
    svg: `<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256" fill="none">
      <g transform="rotate(-14 128 128)">
        <!-- Outer Stamp Rings -->
        <circle cx="128" cy="128" r="102" stroke="#d41414" stroke-width="9" stroke-dasharray="14 7"/>
        <circle cx="128" cy="128" r="88" stroke="#d41414" stroke-width="4"/>
        <circle cx="128" cy="128" r="80" stroke="#d41414" stroke-width="2"/>
        <!-- Star -->
        <polygon points="128,64 133,76 146,76 136,84 140,96 128,88 116,96 120,84 110,76 123,76" fill="#d41414"/>
        <!-- Text -->
        <text x="128" y="146" text-anchor="middle" font-family="'PingFang SC', sans-serif" font-weight="900" font-size="44" fill="#d41414" letter-spacing="4">绝赞</text>
        <text x="128" y="176" text-anchor="middle" font-family="'Arial Black', sans-serif" font-weight="bold" font-size="16" fill="#d41414" letter-spacing="5">APPROVED</text>
      </g>
    </svg>`,
  },
  {
    id: "rejected_stamp",
    name: "驳回印章",
    category: "emotion",
    defaultWidth: 140,
    svg: `<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256" fill="none">
      <g transform="rotate(12 128 128)">
        <!-- Rect Box -->
        <rect x="24" y="68" width="208" height="120" rx="12" stroke="#c91010" stroke-width="9" stroke-dasharray="12 6" fill="none"/>
        <rect x="36" y="80" width="184" height="96" rx="6" stroke="#c91010" stroke-width="3" fill="none"/>
        <!-- Stamp Text -->
        <text x="128" y="142" text-anchor="middle" font-family="'PingFang SC', sans-serif" font-weight="900" font-size="50" fill="#c91010" letter-spacing="8">驳回</text>
        <text x="128" y="164" text-anchor="middle" font-family="'Arial Black', sans-serif" font-weight="900" font-size="15" fill="#c91010" letter-spacing="6">REJECTED</text>
      </g>
    </svg>`,
  },
  {
    id: "anger_vein",
    name: "暴青筋",
    category: "emotion",
    defaultWidth: 80,
    svg: `<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256" fill="none">
      <!-- Comic Anger Symbol (Four-lobed cross) -->
      <path d="
        M 104 36 C 104 74 74 104 36 104 C 36 116 48 116 48 116 C 86 116 116 86 116 48 Z
        M 152 36 C 152 74 182 104 220 104 C 220 116 208 116 208 116 C 170 116 140 86 140 48 Z
        M 36 152 C 74 152 104 182 104 220 C 116 220 116 208 116 208 C 116 170 86 140 48 140 Z
        M 220 152 C 182 152 152 182 152 220 C 140 220 140 208 140 208 C 140 170 170 140 208 140 Z
      " fill="#ff1a1a" stroke="#600000" stroke-width="6" stroke-linejoin="round"/>
      <path d="M 88 128 H 168 M 128 88 V 168" stroke="#ff8080" stroke-width="4" stroke-linecap="round"/>
    </svg>`,
  },
  {
    id: "sweat_drop",
    name: "汗颜汗滴",
    category: "emotion",
    defaultWidth: 70,
    svg: `<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256" fill="none">
      <defs>
        <linearGradient id="dropGrad" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0%" stop-color="#bfe8ff"/>
          <stop offset="50%" stop-color="#33a6ff"/>
          <stop offset="100%" stop-color="#0062cc"/>
        </linearGradient>
      </defs>
      <!-- Teardrop shape -->
      <path d="M 128 24 C 128 24 64 116 64 164 C 64 200 92 232 128 232 C 164 232 192 200 192 164 C 192 116 128 24 128 24 Z" fill="url(#dropGrad)" stroke="#003d80" stroke-width="7"/>
      <!-- Highlights -->
      <path d="M 96 152 C 96 182 108 206 128 206" stroke="#ffffff" stroke-width="8" stroke-linecap="round" fill="none"/>
      <ellipse cx="152" cy="120" rx="8" ry="14" fill="#ffffff" opacity="0.75" transform="rotate(25 152 120)"/>
    </svg>`,
  },
  {
    id: "sparkle_stars",
    name: "崇拜闪光",
    category: "emotion",
    defaultWidth: 100,
    svg: `<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256" fill="none">
      <!-- Big central sparkle -->
      <path d="M 128 16 Q 128 128 16 128 Q 128 128 128 240 Q 128 128 240 128 Q 128 128 128 16 Z" fill="#ffd700" stroke="#7a5500" stroke-width="6"/>
      <circle cx="128" cy="128" r="24" fill="#ffffff"/>
      <!-- Mini top right sparkle -->
      <path d="M 196 28 Q 196 68 156 68 Q 196 68 196 108 Q 196 68 236 68 Q 196 68 196 28 Z" fill="#ffec80" stroke="#7a5500" stroke-width="4"/>
      <!-- Mini bottom left sparkle -->
      <path d="M 52 164 Q 52 192 24 192 Q 52 192 52 220 Q 52 192 80 192 Q 52 192 52 164 Z" fill="#ffec80" stroke="#7a5500" stroke-width="4"/>
    </svg>`,
  },
  {
    id: "heart_eyes",
    name: "爱心桃心眼",
    category: "emotion",
    defaultWidth: 120,
    svg: `<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256" fill="none">
      <defs>
        <radialGradient id="heartGrad" cx="35%" cy="35%" r="70%">
          <stop offset="0%" stop-color="#ff7f99"/>
          <stop offset="60%" stop-color="#ff1a53"/>
          <stop offset="100%" stop-color="#b30030"/>
        </radialGradient>
      </defs>
      <!-- Left Heart -->
      <g transform="translate(16, 68) rotate(-10 50 60)">
        <path d="M 50 20 C 30 -10 0 0 0 35 C 0 70 45 100 50 110 C 55 100 100 70 100 35 C 100 0 70 -10 50 20 Z" fill="url(#heartGrad)" stroke="#5e0018" stroke-width="5"/>
        <ellipse cx="28" cy="28" rx="8" ry="14" fill="#ffffff" opacity="0.8" transform="rotate(-30 28 28)"/>
      </g>
      <!-- Right Heart -->
      <g transform="translate(136, 68) rotate(10 50 60)">
        <path d="M 50 20 C 30 -10 0 0 0 35 C 0 70 45 100 50 110 C 55 100 100 70 100 35 C 100 0 70 -10 50 20 Z" fill="url(#heartGrad)" stroke="#5e0018" stroke-width="5"/>
        <ellipse cx="28" cy="28" rx="8" ry="14" fill="#ffffff" opacity="0.8" transform="rotate(-30 28 28)"/>
      </g>
    </svg>`,
  },
  {
    id: "fire_flame",
    name: "高能烈火",
    category: "emotion",
    defaultWidth: 110,
    svg: `<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256" fill="none">
      <!-- Outer Red/Orange Flame -->
      <path d="M 128 16 C 148 54 186 78 186 122 C 186 136 178 148 170 156 C 182 144 196 128 196 108 C 218 136 226 170 208 206 C 190 236 148 244 128 244 C 98 244 58 230 46 196 C 30 154 56 108 84 80 C 84 104 96 124 110 134 C 110 94 118 48 128 16 Z" fill="#ff3b00" stroke="#6b0d00" stroke-width="5"/>
      <!-- Inner Yellow/Orange Flame -->
      <path d="M 128 88 C 142 114 164 132 164 162 C 164 172 158 180 152 186 C 160 178 170 166 170 152 C 184 172 186 194 174 216 C 162 232 138 236 128 236 C 108 236 84 226 76 206 C 68 180 84 148 102 130 C 102 144 108 156 116 162 C 116 136 122 108 128 88 Z" fill="#ffb700"/>
      <!-- Core White/Yellow Flame -->
      <path d="M 128 160 C 136 174 146 184 146 198 C 146 220 128 230 128 230 C 128 230 110 220 110 198 C 110 184 120 174 128 160 Z" fill="#ffffff"/>
    </svg>`,
  },
  {
    id: "crying_tears",
    name: "面条宽泪",
    category: "emotion",
    defaultWidth: 130,
    svg: `<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256" fill="none">
      <defs>
        <linearGradient id="tearStream" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stop-color="#b8e4ff"/>
          <stop offset="50%" stop-color="#38a8ff"/>
          <stop offset="100%" stop-color="#0068cc"/>
        </linearGradient>
      </defs>
      <!-- Left Waterfall -->
      <g transform="translate(36, 32)">
        <path d="M 20 0 C 40 40 10 100 35 150 C 45 170 15 190 25 210 H -5 C -15 190 15 170 5 150 C -20 100 10 40 -10 0 Z" fill="url(#tearStream)" stroke="#003e80" stroke-width="4"/>
        <ellipse cx="12" cy="60" rx="3" ry="18" fill="#ffffff" opacity="0.8"/>
      </g>
      <!-- Right Waterfall -->
      <g transform="translate(170, 32)">
        <path d="M 20 0 C 40 40 10 100 35 150 C 45 170 15 190 25 210 H -5 C -15 190 15 170 5 150 C -20 100 10 40 -10 0 Z" fill="url(#tearStream)" stroke="#003e80" stroke-width="4"/>
        <ellipse cx="12" cy="60" rx="3" ry="18" fill="#ffffff" opacity="0.8"/>
      </g>
    </svg>`,
  },
  {
    id: "thumbs_up",
    name: "超赞大拇指",
    category: "emotion",
    defaultWidth: 100,
    svg: `<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256" fill="none">
      <g transform="translate(30, 24) rotate(-5 100 100)">
        <!-- Thumb and Hand -->
        <path d="
          M 50 110 V 180 H 20 V 110 Z
          M 50 110 C 50 85 70 30 95 30 C 110 30 115 45 110 65 L 98 100 H 165 C 180 100 185 112 180 124 C 185 130 185 140 178 146 C 182 152 180 162 174 168 C 176 174 172 184 160 184 H 90 C 70 184 50 160 50 140
        " fill="#ffcc00" stroke="#6b4f00" stroke-width="6" stroke-linejoin="round"/>
        <!-- Glove cuff -->
        <rect x="10" y="104" width="30" height="84" rx="6" fill="#ffffff" stroke="#6b4f00" stroke-width="5"/>
        <!-- Finger Creases -->
        <path d="M 120 124 H 168 M 115 146 H 165 M 110 168 H 158" stroke="#cca300" stroke-width="4" stroke-linecap="round"/>
      </g>
    </svg>`,
  },
  {
    id: "shout_bubble",
    name: "尖角咆哮气泡",
    category: "bubble",
    defaultWidth: 160,
    svg: `<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256" fill="none">
      <!-- Jagged explosion comic speech bubble -->
      <polygon points="
        128,18 152,48 188,28 194,66 232,60 220,96 252,118 226,144 246,176 210,184
        214,222 176,212 162,246 132,224 100,248 94,214 56,228 66,192 24,188 44,156
        12,132 44,106 20,74 58,74 62,34 98,52
      " fill="#ffffff" stroke="#111111" stroke-width="7" stroke-linejoin="round"/>
      <!-- Inner accent exclamation marks -->
      <text x="128" y="148" text-anchor="middle" font-family="'Arial Black', sans-serif" font-weight="900" font-size="44" fill="#ff2222">！ ！</text>
    </svg>`,
  },
  {
    id: "dots_bubble",
    name: "无语省略号气泡",
    category: "bubble",
    defaultWidth: 150,
    svg: `<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256" fill="none">
      <!-- Smooth comic bubble -->
      <path d="
        M 128 36
        C 188 36 236 72 236 118
        C 236 164 188 200 128 200
        C 106 200 86 194 68 184
        L 28 208
        L 42 168
        C 28 154 20 136 20 118
        C 20 72 68 36 128 36 Z
      " fill="#ffffff" stroke="#1a1a1a" stroke-width="7" stroke-linejoin="round"/>
      <!-- Three dots -->
      <circle cx="88" cy="118" r="10" fill="#1a1a1a"/>
      <circle cx="128" cy="118" r="10" fill="#1a1a1a"/>
      <circle cx="168" cy="118" r="10" fill="#1a1a1a"/>
    </svg>`,
  },
];

console.log(`Generating ${STICKERS_METADATA.length} stickers...`);

for (const item of STICKERS_METADATA) {
  const svgPath = path.join(outputDir, `${item.id}.svg`);
  const pngPath = path.join(outputDir, `${item.id}.png`);

  fs.writeFileSync(svgPath, item.svg.trim(), "utf8");

  try {
    execFileSync("sips", ["-s", "format", "png", svgPath, "--out", pngPath], {
      stdio: "pipe",
    });
    // Remove intermediate svg
    fs.unlinkSync(svgPath);
    console.log(`✓ Generated ${item.id}.png (${item.name})`);
  } catch (error) {
    console.error(`✗ Failed to generate ${item.id}:`, error);
    process.exit(1);
  }
}

console.log(`All ${STICKERS_METADATA.length} stickers successfully generated in ${outputDir}!`);

#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.resolve(__dirname, '..');
const binariesDir = path.resolve(projectRoot, 'src-tauri', 'binaries');

if (!fs.existsSync(binariesDir)) {
  fs.mkdirSync(binariesDir, { recursive: true });
}

// Target triples configuration
const TARGET_MAP = {
  'darwin-arm64': 'aarch64-apple-darwin',
  'darwin-x64': 'x86_64-apple-darwin',
  'win32-x64': 'x86_64-pc-windows-msvc',
  'linux-x64': 'x86_64-unknown-linux-gnu',
};

const PLATFORM_SOURCES = {
  'aarch64-apple-darwin': {
    ffmpeg: 'https://ffmpeg.martin-riedl.de/download/macos/arm64/1789931890_9.0.2/ffmpeg.zip',
    ffprobe: 'https://ffmpeg.martin-riedl.de/download/macos/arm64/1789931890_9.0.2/ffprobe.zip',
    ext: '',
  },
  'x86_64-apple-darwin': {
    ffmpeg: 'https://ffmpeg.martin-riedl.de/download/macos/amd64/1789931006_9.0.2/ffmpeg.zip',
    ffprobe: 'https://ffmpeg.martin-riedl.de/download/macos/amd64/1789931006_9.0.2/ffprobe.zip',
    ext: '',
  },
  'x86_64-pc-windows-msvc': {
    combined: 'https://github.com/yt-dlp/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip',
    ext: '.exe',
  },
  'x86_64-unknown-linux-gnu': {
    ffmpeg: 'https://ffmpeg.martin-riedl.de/download/linux/amd64/1789931100_9.0.2/ffmpeg.zip',
    ffprobe: 'https://ffmpeg.martin-riedl.de/download/linux/amd64/1789931100_9.0.2/ffprobe.zip',
    ext: '',
  },
};

function getHostTarget() {
  const key = `${process.platform}-${process.arch}`;
  const target = TARGET_MAP[key];
  if (!target) {
    throw new Error(`Unsupported host platform/architecture: ${key}`);
  }
  return target;
}

async function downloadFile(url, dest) {
  console.log(`Downloading: ${url}`);
  const response = await fetch(url, { redirect: 'follow' });
  if (!response.ok) {
    throw new Error(`Failed to download ${url}: ${response.status} ${response.statusText}`);
  }
  const arrayBuffer = await response.arrayBuffer();
  fs.writeFileSync(dest, Buffer.from(arrayBuffer));
  console.log(`Saved to ${dest} (${(arrayBuffer.byteLength / 1024 / 1024).toFixed(2)} MB)`);
}

function extractZip(zipPath, outDir) {
  console.log(`Extracting ${zipPath}...`);
  if (process.platform === 'win32') {
    execFileSync('powershell', ['-NoProfile', '-Command', `Expand-Archive -Path '${zipPath}' -DestinationPath '${outDir}' -Force`]);
  } else {
    execFileSync('unzip', ['-o', zipPath, '-d', outDir]);
  }
}

function findBinary(dir, name) {
  const files = fs.readdirSync(dir, { recursive: true });
  for (const file of files) {
    const base = path.basename(file);
    if (base === name || base === `${name}.exe`) {
      return path.join(dir, file);
    }
  }
  return null;
}

async function prepareTarget(target) {
  const config = PLATFORM_SOURCES[target];
  if (!config) {
    throw new Error(`No source configuration for target: ${target}`);
  }

  const ffmpegDestName = `ffmpeg-${target}${config.ext}`;
  const ffprobeDestName = `ffprobe-${target}${config.ext}`;
  const ffmpegFinalPath = path.join(binariesDir, ffmpegDestName);
  const ffprobeFinalPath = path.join(binariesDir, ffprobeDestName);

  if (fs.existsSync(ffmpegFinalPath) && fs.existsSync(ffprobeFinalPath)) {
    console.log(`✓ Binaries for ${target} already exist in src-tauri/binaries/`);
    verifyBinaries(ffmpegFinalPath, ffprobeFinalPath);
    return;
  }

  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), `ffmpeg-sidecar-${target}-`));

  try {
    if (config.combined) {
      const zipPath = path.join(tmpDir, 'bundle.zip');
      await downloadFile(config.combined, zipPath);
      extractZip(zipPath, tmpDir);

      const foundFfmpeg = findBinary(tmpDir, 'ffmpeg');
      const foundFfprobe = findBinary(tmpDir, 'ffprobe');
      if (!foundFfmpeg || !foundFfprobe) {
        throw new Error(`Could not find ffmpeg or ffprobe in ${config.combined}`);
      }
      fs.copyFileSync(foundFfmpeg, ffmpegFinalPath);
      fs.copyFileSync(foundFfprobe, ffprobeFinalPath);
    } else {
      // Download ffmpeg
      const ffmpegZip = path.join(tmpDir, 'ffmpeg.zip');
      const ffmpegExtracted = path.join(tmpDir, 'ffmpeg_out');
      await downloadFile(config.ffmpeg, ffmpegZip);
      extractZip(ffmpegZip, ffmpegExtracted);
      const foundFfmpeg = findBinary(ffmpegExtracted, 'ffmpeg');
      if (!foundFfmpeg) throw new Error('ffmpeg binary not found in archive');
      fs.copyFileSync(foundFfmpeg, ffmpegFinalPath);

      // Download ffprobe
      const ffprobeZip = path.join(tmpDir, 'ffprobe.zip');
      const ffprobeExtracted = path.join(tmpDir, 'ffprobe_out');
      await downloadFile(config.ffprobe, ffprobeZip);
      extractZip(ffprobeZip, ffprobeExtracted);
      const foundFfprobe = findBinary(ffprobeExtracted, 'ffprobe');
      if (!foundFfprobe) throw new Error('ffprobe binary not found in archive');
      fs.copyFileSync(foundFfprobe, ffprobeFinalPath);
    }

    if (process.platform !== 'win32') {
      fs.chmodSync(ffmpegFinalPath, 0o755);
      fs.chmodSync(ffprobeFinalPath, 0o755);
    }

    console.log(`✓ Successfully installed ${ffmpegDestName} and ${ffprobeDestName}`);
    verifyBinaries(ffmpegFinalPath, ffprobeFinalPath);
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
}

function verifyBinaries(ffmpegPath, ffprobePath) {
  if (process.platform === 'darwin' && os.arch() === 'arm64' && ffmpegPath.includes('aarch64-apple-darwin')) {
    try {
      const versionOutput = execFileSync(ffmpegPath, ['-version'], { encoding: 'utf-8' });
      const firstLine = versionOutput.split('\n')[0];
      console.log(`[Verify] FFmpeg Version: ${firstLine}`);

      const filtersOutput = execFileSync(ffmpegPath, ['-filters'], { encoding: 'utf-8' });
      const hasDrawtext = filtersOutput.includes('drawtext');
      console.log(`[Verify] Has drawtext filter (libfreetype): ${hasDrawtext ? 'YES ✓' : 'NO ✗'}`);
      if (!hasDrawtext) {
        throw new Error('Downloaded binary lacks drawtext support!');
      }

      const probeOutput = execFileSync(ffprobePath, ['-version'], { encoding: 'utf-8' });
      console.log(`[Verify] FFprobe Version: ${probeOutput.split('\n')[0]}`);
    } catch (err) {
      console.error('[Verify Failed]', err.message);
      throw err;
    }
  }
}

async function main() {
  const args = process.argv.slice(2);
  const targetArg = args.find((a) => a.startsWith('--target='));
  const target = targetArg ? targetArg.split('=')[1] : getHostTarget();

  console.log(`=== FFmpeg Sidecar Setup ===`);
  console.log(`Target: ${target}`);
  console.log(`Destination: ${binariesDir}`);

  await prepareTarget(target);
  console.log(`Done!`);
}

main().catch((err) => {
  console.error(`Error:`, err.message);
  process.exit(1);
});

import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');
const outputDir = path.join(rootDir, 'src-tauri', 'installer-assets');
const requiredAssets = ['header.bmp', 'sidebar.bmp'].map((file) =>
  path.join(outputDir, file)
);

if (process.platform !== 'win32') {
  const missingAssets = requiredAssets.filter((file) => !fs.existsSync(file));

  if (missingAssets.length > 0) {
    console.error(
      `Installer assets are missing and cannot be generated automatically on ${process.platform}.`
    );
    process.exit(1);
  }

  console.log(
    `Skipping installer asset generation on ${process.platform}; using committed BMP assets.`
  );
  process.exit(0);
}

const powershellPath =
  process.env.SystemRoot == null
    ? 'powershell.exe'
    : path.join(
        process.env.SystemRoot,
        'System32',
        'WindowsPowerShell',
        'v1.0',
        'powershell.exe'
      );

const scriptPath = path.join(__dirname, 'generate-installer-assets.ps1');
const result = spawnSync(
  powershellPath,
  ['-ExecutionPolicy', 'Bypass', '-File', scriptPath],
  {
    cwd: rootDir,
    stdio: 'inherit'
  }
);

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 0);

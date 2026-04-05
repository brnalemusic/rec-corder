import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

function readVersion() {
  const versionPath = path.join(rootDir, 'version.txt');
  if (fs.existsSync(versionPath)) {
    return fs.readFileSync(versionPath, 'utf8').trim();
  }

  const pkgPath = path.join(rootDir, 'package.json');
  if (fs.existsSync(pkgPath)) {
    const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));
    if (pkg.version) {
      return pkg.version;
    }
  }

  console.error('version.txt not found at root, and package.json has no version.');
  process.exit(1);
}

function updateFile(file, transform) {
  const fullPath = path.join(rootDir, file);
  if (!fs.existsSync(fullPath)) {
    console.warn(`Arquivo nao encontrado: ${file}`);
    return;
  }

  const content = fs.readFileSync(fullPath, 'utf8');
  const nextContent = transform(content);

  if (content === nextContent) {
    console.log(`Sem mudancas: ${file}`);
    return;
  }

  fs.writeFileSync(fullPath, nextContent, 'utf8');
  console.log(`Atualizado: ${file}`);
}

function syncJsonVersion(file, version, extraMutator) {
  updateFile(file, (content) => {
    const json = JSON.parse(content);
    json.version = version;

    if (extraMutator) {
      extraMutator(json, version);
    }

    return `${JSON.stringify(json, null, 2)}\n`;
  });
}

function syncCargoLockVersion(version) {
  updateFile('src-tauri/Cargo.lock', (content) =>
    content.replace(
      /(\[\[package\]\]\r?\nname = "rec-corder"\r?\nversion = ")[^"]+(")/,
      `$1${version}$2`
    )
  );
}

function syncTextTargets(version) {
  syncJsonVersion('package.json', version);
  syncJsonVersion('package-lock.json', version, (json) => {
    if (json.packages?.['']) {
      json.packages[''].version = version;
    }
  });

  updateFile('src-tauri/tauri.conf.json', (content) =>
    content.replace(/"version":\s*"[^"]*"/, `"version": "${version}"`)
  );

  updateFile('src-tauri/Cargo.toml', (content) =>
    content.replace(/^version\s*=\s*"[^"]*"/m, `version = "${version}"`)
  );

  syncCargoLockVersion(version);

  updateFile('src-tauri/nsis_hook.nsh', (content) =>
    content.replace(/Rec Corder v[0-9.-a-z]+/gi, `Rec Corder v${version}`)
  );

  updateFile('README.md', (content) =>
    content.replace(
      /sub>Criado com .* para a comunidade criativa\. v[0-9.-a-z]+<\/sub>/i,
      `sub>Criado com 🧡 para a comunidade criativa. v${version}</sub>`
    )
  );
}

function syncInstallerAssets() {
  const installerAssetsScript = path.join(
    rootDir,
    'scripts',
    'generate-installer-assets.js'
  );

  if (!fs.existsSync(installerAssetsScript)) {
    console.warn(
      'Script de assets do instalador nao encontrado: scripts/generate-installer-assets.js'
    );
    return;
  }

  console.log('Regenerando assets do instalador...');
  const installerAssets = spawnSync(process.execPath, [installerAssetsScript], {
    cwd: rootDir,
    stdio: 'inherit'
  });

  if (installerAssets.status !== 0) {
    console.error('Falha ao regenerar os assets do instalador.');
    process.exit(installerAssets.status ?? 1);
  }
}

function sync() {
  const version = readVersion();
  console.log(`Sincronizando versao: ${version}`);

  syncTextTargets(version);
  syncInstallerAssets();

  console.log('Sincronizacao concluida com sucesso.');
}

sync();

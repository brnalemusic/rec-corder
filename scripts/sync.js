import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

function readVersions() {
  const versionPath = path.join(rootDir, 'version.txt');
  if (!fs.existsSync(versionPath)) {
    console.error('Erro: version.txt nao encontrado na raiz.');
    process.exit(1);
  }

  const rawContent = fs.readFileSync(versionPath, 'utf8').trim();
  // Formato esperado: "1.1.0-beta.1 | cli0.1.0"
  const parts = rawContent.split('|').map(p => p.trim());

  if (parts.length !== 2 || !parts[1].startsWith('cli')) {
    console.error('Erro: Formato de version.txt invalido.');
    console.error('Formato correto: "VersaoApp | cliVersaoCli"');
    console.error('Exemplo: "1.0.0-beta | cli0.1.0"');
    process.exit(1);
  }

  const appVersion = parts[0];
  const cliVersion = parts[1].replace(/^cli/, '');

  return { appVersion, cliVersion };
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

function syncTextTargets(appVersion, cliVersion) {
  syncJsonVersion('package.json', appVersion);
  syncJsonVersion('package-lock.json', appVersion, (json) => {
    if (json.packages?.['']) {
      json.packages[''].version = appVersion;
    }
  });

  updateFile('src-tauri/tauri.conf.json', (content) =>
    content.replace(/"version":\s*"[^"]*"/, `"version": "${appVersion}"`)
  );

  updateFile('src-tauri/Cargo.toml', (content) =>
    content.replace(/^version\s*=\s*"[^"]*"/m, `version = "${appVersion}"`)
  );

  syncCargoLockVersion(appVersion);

  updateFile('src-tauri/nsis_hook.nsh', (content) =>
    content.replace(/Rec Corder v[^"]+/gi, `Rec Corder v${appVersion}`)
  );

  updateFile('README.md', (content) => {
    let newContent = content.replace(
      /sub>Criado com .* para a comunidade criativa\. v[0-9a-z.-]+<\/sub>/i,
      `sub>Criado com 🧡 para a comunidade criativa. v${appVersion}</sub>`
    );
    return newContent;
  });

  updateFile('cli/main.py', (content) =>
    content.replace(
      /Ultra-light Screen Recorder — v[0-9a-z.-]+/i,
      `Ultra-light Screen Recorder — v${cliVersion}`
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
  const { appVersion, cliVersion } = readVersions();
  console.log(`Sincronizando: App v${appVersion} | CLI v${cliVersion}`);

  syncTextTargets(appVersion, cliVersion);
  syncInstallerAssets();

  console.log('Verificando dependencias do CLI (Python)...');
  spawnSync(process.execPath, [path.join(rootDir, 'scripts', 'download-python.js')], { stdio: 'inherit' });

  console.log('Sincronizacao concluida com sucesso.');
}

sync();

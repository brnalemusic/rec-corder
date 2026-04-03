import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

function sync() {
  const versionPath = path.join(rootDir, 'version.txt');
  if (!fs.existsSync(versionPath)) {
    console.error('❌ version.txt not found at root.');
    process.exit(1);
  }

  const version = fs.readFileSync(versionPath, 'utf8').trim();
  console.log(`🚀 Sincronizando versão: ${version}`);

  const targets = [
    {
      file: 'package.json',
      regex: /"version":\s*"[^"]*"/,
      replace: `"version": "${version}"`
    },
    {
      file: 'src-tauri/tauri.conf.json',
      regex: /"version":\s*"[^"]*"/,
      replace: `"version": "${version}"`
    },
    {
      file: 'src-tauri/Cargo.toml',
      regex: /^version\s*=\s*"[^"]*"/m,
      replace: `version = "${version}"`
    },
    {
      file: 'src-tauri/src/commands/recorder.rs',
      regex: /version:\s*"[^"]*"\.to_string\(\)/,
      replace: `version: "${version}".to_string()`
    },
    {
      file: 'src-tauri/pre_install.ps1',
      regex: /Rec Corder v[0-9.-a-z]+|FFmpeg v[0-9.-a-z]+/gi,
      replace: (match) => match.includes('Rec Corder') ? `Rec Corder v${version}` : `FFmpeg v${version}`
    },
    {
      file: 'src-tauri/nsis_hook.nsh',
      regex: /Rec Corder v[0-9.-a-z]+/gi,
      replace: `Rec Corder v${version}`
    },
    {
      file: 'README.md',
      regex: /sub>Criado com 🧡 para a comunidade criativa\. v[0-9.-a-z]+<\/sub>/i,
      replace: `sub>Criado com 🧡 para a comunidade criativa. v${version}</sub>`
    }
  ];

  targets.forEach(target => {
    const fullPath = path.join(rootDir, target.file);
    if (fs.existsSync(fullPath)) {
      let content = fs.readFileSync(fullPath, 'utf8');
      const newContent = content.replace(target.regex, target.replace);
      
      if (content !== newContent) {
        fs.writeFileSync(fullPath, newContent, 'utf8');
        console.log(`✅ Atualizado: ${target.file}`);
      } else {
        console.log(`ℹ️ Sem mudanças: ${target.file} (já está na versão ${version})`);
      }
    } else {
      console.warn(`⚠️ Arquivo não encontrado: ${target.file}`);
    }
  });

  console.log('✨ Sincronização concluída com sucesso!');
}

sync();

import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { execSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');
const srcTauri = path.join(rootDir, 'src-tauri');
const cliDir = path.join(rootDir, 'cli');

// Usar um diretorio totalmente fora do workspace para o Rust resolver problemas de file lock no Windows (Antivirus/OneDrive)
const tmpTargetDir = path.join(os.tmpdir(), 'rec-corder-cli-target');

console.log('Construindo backend Rust para o CLI (PyO3) em diretorio externo...');
try {
    execSync('cargo build --lib --features python --release', { 
        cwd: srcTauri, 
        stdio: 'inherit',
        env: { ...process.env, CARGO_TARGET_DIR: tmpTargetDir }
    });

    const isWin = process.platform === 'win32';
    const libExt = isWin ? 'dll' : 'so';
    const libPrefix = isWin ? '' : 'lib';
    const outputExt = isWin ? 'pyd' : 'so';

    const libPath = path.join(tmpTargetDir, 'release', `${libPrefix}rec_corder_lib.${libExt}`);
    const destPath = path.join(cliDir, `rec_corder_lib.${outputExt}`);

    if (fs.existsSync(libPath)) {
        fs.copyFileSync(libPath, destPath);
        console.log(`Modulo nativo gerado com sucesso em: ${destPath}`);
    } else {
        console.error('Biblioteca nao encontrada apos o build em: ' + libPath);
        process.exit(1);
    }
    } catch (e) {    console.error('Falha ao compilar o CLI:', e.message);
    process.exit(1);
}

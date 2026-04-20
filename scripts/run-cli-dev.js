import { execSync, spawn } from 'node:child_process';
import path from 'node:path';
import fs from 'node:fs';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

const pythonExe = path.join(rootDir, 'src-tauri', 'python_env', 'python.exe');
const cliMain = path.join(rootDir, 'cli', 'main.py');
const pydFile = path.join(rootDir, 'cli', 'rec_corder_lib.pyd');
const srcTauri = path.join(rootDir, 'src-tauri');

/**
 * [CACHE INTELIGENTE]
 * Obtém a data da última modificação de todos os arquivos do backend Rust.
 */
function getLatestRustMTime(dir) {
    let latest = 0;
    const files = fs.readdirSync(dir);

    for (const file of files) {
        const fullPath = path.join(dir, file);
        const stat = fs.statSync(fullPath);

        if (stat.isDirectory()) {
            latest = Math.max(latest, getLatestRustMTime(fullPath));
        } else if (file.endsWith('.rs') || file.endsWith('.toml')) {
            latest = Math.max(latest, stat.mtimeMs);
        }
    }
    return latest;
}

async function run() {
    console.log('--- Rec Corder CLI Dev (Com Cache) ---');

    // 1. Verificar ambiente Python
    if (!fs.existsSync(pythonExe)) {
        console.log('Ambiente Python não encontrado. Baixando...');
        execSync('node scripts/download-python.js', { stdio: 'inherit', cwd: rootDir });
    }

    // 2. Lógica de Cache para o Build
    let needBuild = true;
    if (fs.existsSync(pydFile)) {
        const pydMTime = fs.statSync(pydFile).mtimeMs;
        const lastSrcMTime = getLatestRustMTime(srcTauri);

        if (pydMTime > lastSrcMTime) {
            console.log('\x1b[32m%s\x1b[0m', '>> Cache: Nenhuma alteração detectada no Rust. Pulando build...');
            needBuild = false;
        } else {
            console.log('\x1b[33m%s\x1b[0m', '>> Alterações detectadas no backend. Iniciando build incremental...');
        }
    }

    if (needBuild) {
        try {
            // Executa o build apenas se necessário
            execSync('node scripts/build-cli.js', { stdio: 'inherit', cwd: rootDir });
        } catch (e) {
            console.error('Erro no build do Rust. Verifique os logs.');
            process.exit(1);
        }
    }

    // 3. Abrir em um novo terminal
    console.log('Abrindo CLI em uma nova janela...');
    
    if (process.platform === 'win32') {
        // start /D (diretório de trabalho) "Título" "Executável" "Script"
        const command = `start "Rec Corder CLI - Modo Dev" /D "${path.dirname(cliMain)}" "${pythonExe}" "${cliMain}"`;
        execSync(command, { shell: true });
    } else {
        spawn(pythonExe, [cliMain], { stdio: 'inherit' });
    }

    console.log('Pronto!');
}

run().catch(err => {
    console.error('Falha ao iniciar CLI dev:', err);
    process.exit(1);
});

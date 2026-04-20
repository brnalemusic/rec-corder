import fs from 'node:fs';
import path from 'node:path';
import { execSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const rootDir = path.resolve(__dirname, '..');
const pythonEnvDir = path.join(rootDir, 'src-tauri', 'python_env');

// Configurações de hardening
const PYTHON_VERSION = '3.11.9';
const PYTHON_URL = `https://www.python.org/ftp/python/${PYTHON_VERSION}/python-${PYTHON_VERSION}-embed-amd64.zip`;
const EXPECTED_SHA256 = '009d6bf7e3b2ddca3d784fa09f90fe54336d5b60f0e0f305c37f400bf83cfd3b';
const MAX_ATTEMPTS = 3;

async function setupPython() {
    if (fs.existsSync(pythonEnvDir)) {
        console.log('Python portatil ja configurado em src-tauri/python_env.');
        return;
    }

    console.log(`Configurando ambiente Python ${PYTHON_VERSION}...`);
    fs.mkdirSync(pythonEnvDir, { recursive: true });

    if (process.platform !== 'win32') {
        console.log('Plataforma nao-Windows detectada. Assumindo uso do Python do sistema.');
        console.log('Pasta python_env criada para satisfazer o bundler do Tauri.');
        return;
    }

    const zipPath = path.join(pythonEnvDir, 'python.zip');
    let success = false;

    for (let attempt = 1; attempt <= MAX_ATTEMPTS; attempt++) {
        try {
            console.log(`Tentativa ${attempt}/${MAX_ATTEMPTS}: Baixando Python portatil...`);
            
            // Download em uma única linha para evitar erros de parser do PowerShell via execSync
            const downloadCmd = `powershell -NoProfile -Command "$ProgressPreference = 'SilentlyContinue'; Invoke-WebRequest -Uri '${PYTHON_URL}' -OutFile '${zipPath}' -TimeoutSec 180"`;
            execSync(downloadCmd, { stdio: 'inherit' });

            console.log('Verificando integridade (SHA-256)...');
            const hashCmd = `powershell -NoProfile -Command "(Get-FileHash -Path '${zipPath}' -Algorithm SHA256).Hash"`;
            const actualHash = execSync(hashCmd).toString().trim().toLowerCase();

            if (actualHash === EXPECTED_SHA256.toLowerCase()) {
                console.log('Integridade confirmada.');
                success = true;
                break;
            } else {
                console.error(`ERRO: Hash incorreto! Esperado: ${EXPECTED_SHA256}, Obtido: ${actualHash}`);
                if (fs.existsSync(zipPath)) fs.unlinkSync(zipPath);
            }
        } catch (e) {
            console.error(`Falha na tentativa ${attempt}: ${e.message}`);
            if (fs.existsSync(zipPath)) fs.unlinkSync(zipPath);
            if (attempt < MAX_ATTEMPTS) {
                console.log('Aguardando 5 segundos para nova tentativa...');
                execSync('powershell -NoProfile -Command "Start-Sleep -Seconds 5"');
            }
        }
    }

    if (!success) {
        console.error('Nao foi possivel baixar o Python apos varias tentativas ou falha de integridade.');
        process.exit(1);
    }

    try {
        console.log('Extraindo Python portatil...');
        execSync(`powershell -NoProfile -Command "Expand-Archive -Path '${zipPath}' -DestinationPath '${pythonEnvDir}' -Force"`, { stdio: 'inherit' });
        fs.unlinkSync(zipPath);

        // Modifica o ._pth para permitir importacao de modulos locais do python_env
        const pthFile = path.join(pythonEnvDir, `python${PYTHON_VERSION.split('.').slice(0, 2).join('')}._pth`);
        if (fs.existsSync(pthFile)) {
            let content = fs.readFileSync(pthFile, 'utf8');
            content = content.replace(/#import site/, 'import site');
            fs.writeFileSync(pthFile, content);
        }
        console.log('Python portatil configurado com sucesso.');
    } catch (e) {
        console.error('Falha ao extrair/configurar Python portatil:', e.message);
        process.exit(1);
    }
}

setupPython();

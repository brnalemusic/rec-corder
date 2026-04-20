import fs from 'node:fs';
import path from 'node:path';
import { execSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const rootDir = path.resolve(__dirname, '..');
const pythonEnvDir = path.join(rootDir, 'src-tauri', 'python_env');

if (!fs.existsSync(pythonEnvDir)) {
    console.log('Configurando ambiente Python...');
    fs.mkdirSync(pythonEnvDir, { recursive: true });

    if (process.platform === 'win32') {
        console.log('Baixando Python portatil para Windows...');
        const url = 'https://www.python.org/ftp/python/3.11.9/python-3.11.9-embed-amd64.zip';
        const zipPath = path.join(pythonEnvDir, 'python.zip');

        try {
            execSync(`powershell -NoProfile -Command "Invoke-WebRequest -Uri '${url}' -OutFile '${zipPath}'"`, { stdio: 'inherit' });
            console.log('Extraindo Python portatil...');
            execSync(`powershell -NoProfile -Command "Expand-Archive -Path '${zipPath}' -DestinationPath '${pythonEnvDir}' -Force"`, { stdio: 'inherit' });
            fs.unlinkSync(zipPath);

            // Modifica o ._pth para permitir importacao de modulos locais do python_env
            const pthFile = path.join(pythonEnvDir, 'python311._pth');
            if (fs.existsSync(pthFile)) {
                let content = fs.readFileSync(pthFile, 'utf8');
                content = content.replace(/#import site/, 'import site');
                fs.writeFileSync(pthFile, content);
            }
            console.log('Python portatil configurado com sucesso.');
        } catch (e) {
            console.error('Falha ao configurar Python portatil:', e.message);
            process.exit(1);
        }
    } else {
        console.log('Plataforma nao-Windows detectada. Assumindo uso do Python do sistema.');
        console.log('Pasta python_env criada para satisfazer o bundler do Tauri.');
    }
} else {    console.log('Python portatil ja configurado.');
}

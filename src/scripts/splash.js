const { invoke } = window.__TAURI__.core;

async function bootstrap() {
  const statusText = document.getElementById('statusText');
  const splashVersion = document.getElementById('splashVersion');

  try {
    const version = await window.__TAURI__.app.getVersion();
    if (splashVersion) splashVersion.textContent = `v${version}`;
  } catch (e) {
    console.warn('Failed to get version for splash:', e);
  }

  try {
    // Verificar se FFmpeg está disponível
    const ffmpegStatus = await invoke('check_ffmpeg');
    
    if (!ffmpegStatus.found) {
      console.warn('FFmpeg não encontrado.');
      statusText.style.color = '#ff4444';
      statusText.textContent = 'Erro: FFmpeg não encontrado';
      await new Promise(r => setTimeout(r, 2000));
      return; // Interrompe caso o FFmpeg não seja detectado
    } else {
      console.log('FFmpeg encontrado em:', ffmpegStatus.path);
      statusText.style.color = '#00aa88';
      statusText.textContent = 'FFmpeg embutido detectado ✓';
      await new Promise(r => setTimeout(r, 600));
    }

    // Testar ambiente de encoding
    statusText.style.color = '#ffaa00';
    statusText.textContent = 'Detectando acelerador...';
    const encoder = await invoke('test_environment');
    console.log('Ambiente testado. Encoder:', encoder);

    statusText.style.color = '#00ff88';
    statusText.textContent = `Acelerador: ${encoder}`;

    await new Promise(r => setTimeout(r, 800));
  } catch (err) {
    console.error('Falha na deteccao:', err);
    statusText.style.color = '#ffaa00';
    statusText.textContent = 'Fallback: software (libx264)';

    await new Promise(r => setTimeout(r, 800));
  }

  // Transição segura feita inteiramente pelo Rust (sem depender da API JS de janela)
  try {
    await invoke('finish_splash');
  } catch (e) {
    console.error('Erro na transicao de janela:', e);
  }
}

document.addEventListener('DOMContentLoaded', bootstrap);

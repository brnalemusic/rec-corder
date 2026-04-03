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
      console.warn('FFmpeg não encontrado. Tentando baixar...');
      statusText.style.color = '#ffaa00';
      statusText.textContent = 'Verificando FFmpeg...';
      
      // Aguarda um pouco para o usuário ver a mensagem
      await new Promise(r => setTimeout(r, 800));
      
      try {
        statusText.textContent = 'Baixando FFmpeg (pode demorar)...';
        const result = await invoke('download_ffmpeg');
        console.log('FFmpeg baixado:', result);
        statusText.style.color = '#00ff88';
        statusText.textContent = 'FFmpeg instalado!';
        await new Promise(r => setTimeout(r, 1200));
      } catch (err) {
        console.error('Erro ao baixar FFmpeg:', err);
        statusText.style.color = '#ff4444';
        statusText.textContent = 'Falha ao baixar FFmpeg';
        await new Promise(r => setTimeout(r, 2000));
      }
    } else {
      console.log('FFmpeg encontrado em:', ffmpegStatus.path);
      statusText.style.color = '#00aa88';
      statusText.textContent = 'FFmpeg detectado ✓';
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

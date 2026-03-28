const { invoke } = window.__TAURI__.core;

async function bootstrap() {
  const statusText = document.getElementById('statusText');

  try {
    const encoder = await invoke('test_environment');
    console.log('Ambiente testado. Encoder:', encoder);

    statusText.style.color = '#00ff88';
    statusText.textContent = `Acelerador: ${encoder}`;

    await new Promise(r => setTimeout(r, 600));
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

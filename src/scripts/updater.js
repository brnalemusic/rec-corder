const { invoke } = window.__TAURI__.core;
const { listen, emit } = window.__TAURI__.event;
const { getCurrentWindow } = window.__TAURI__.window;

const appWindow = getCurrentWindow();

// Elements
const btnCancel = document.getElementById('btn-cancel');
const btnInstall = document.getElementById('btn-install');
const statusText = document.getElementById('status-text');
const newVersionSpan = document.getElementById('new-version');
const downloadSection = document.getElementById('download-section');
const progressFill = document.getElementById('progress-fill');
const downloadPercent = document.getElementById('download-percent');
const downloadStatus = document.getElementById('download-status');
const changelogContainer = document.getElementById('changelog-container');
const changelogContent = document.getElementById('changelog-content');

// Listen for data from the backend
let unlistenData;

async function init() {
  // Configure marked for tables and GFM
  if (window.marked) {
    window.marked.setOptions({
      gfm: true,
      breaks: true,
      tables: true
    });
  }

  unlistenData = await listen('updater-data', (event) => {
    const [version, body] = event.payload;
    
    if (version) {
      newVersionSpan.textContent = `v${version}`;
    }

    if (body) {
      changelogContainer.classList.remove('hidden');
      
      let processedBody = body;
      
      // Simple emoji replacement support
      const emojiMap = {
        ':white_check_mark:': '✅',
        ':sparkles:': '✨',
        ':rocket:': '🚀',
        ':bug:': '🐞',
        ':memo:': '📝',
        ':warning:': '⚠️',
        ':x:': '❌',
        ':lock:': '🔒',
        ':tada:': '🎉',
        ':link:': '🔗'
      };
      
      Object.keys(emojiMap).forEach(key => {
        processedBody = processedBody.replace(new RegExp(key, 'g'), emojiMap[key]);
      });

      if (window.marked) {
        changelogContent.innerHTML = window.marked.parse(processedBody);
        
        // Trigger Prism highlighting
        if (window.Prism) {
          window.Prism.highlightAllUnder(changelogContent);
        }
      } else {
        changelogContent.textContent = processedBody;
      }
    }

    // Now that everything is rendered, show the window
    appWindow.show();
    appWindow.setFocus();
  });

  // Notifica o backend que o frontend está pronto para receber os dados
  await emit('updater-ready');
}

init();

btnCancel.addEventListener('click', async () => {
  await emit('updater-close');
});

btnInstall.addEventListener('click', async () => {
  try {
    btnInstall.disabled = true;
    btnCancel.disabled = true;
    
    downloadSection.classList.remove('hidden');
    changelogContainer.classList.add('hidden'); 
    statusText.textContent = 'Baixando atualização... Por favor, não feche o aplicativo.';
    
    let downloadTotal = 0;
    let downloadedBytes = 0;

    const unlistenProgress = await listen('update-progress', (event) => {
      const payload = event.payload;
      const chunk = payload.chunk;
      downloadTotal = payload.total || downloadTotal;
      downloadedBytes += chunk;

      if (downloadTotal > 0) {
        const percent = Math.round((downloadedBytes / downloadTotal) * 100);
        progressFill.style.width = `${percent}%`;
        downloadPercent.textContent = `${percent}%`;
        downloadStatus.textContent = `Baixando (${percent}%)`;
      }
    });

    const unlistenFinished = await listen('update-finished', () => {
      downloadStatus.textContent = 'Instalando...';
      statusText.textContent = 'A atualização foi baixada e está sendo aplicada. O aplicativo será reiniciado em instantes.';
      progressFill.style.width = '100%';
      downloadPercent.textContent = '100%';
    });

    const unlistenError = await listen('update-error', (event) => {
      showError(`Falha técnica: ${event.payload}`);
    });

    await invoke('install_update');

    // If install_update returns without error but doesn't restart, we wait
    // The actual restart is handled by the backend task

  } catch (error) {
    showError(error);
  }
});

function showError(error) {
  console.error('Update failed:', error);
  
  statusText.innerHTML = `
    <div class="error-text">
      Não foi possível concluir a atualização automática.<br>
      Por favor, baixe a versão mais recente manualmente em: 
      <span class="download-link" onclick="window.__TAURI__.core.invoke('open_link', { url: 'https://www.reccorder.com.br' })">www.reccorder.com.br</span>
    </div>
  `;
  
  btnInstall.classList.add('hidden');
  btnCancel.textContent = 'Fechar';
  btnCancel.disabled = false;
  downloadSection.classList.add('hidden');
}

// Cleanup
window.addEventListener('unload', () => {
  if (unlistenData) unlistenData();
});

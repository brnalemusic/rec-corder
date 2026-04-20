/**
 * Rec Corder — Notas de Lançamento
 * Lida com a busca e renderização das notas de lançamento via Markdown.
 */

import * as recorder from './recorder.js';

// Elementos
const versionSubtitle = document.getElementById('version-subtitle');
const loading = document.getElementById('loading');
const contentArea = document.getElementById('content-area');
const changelogContent = document.getElementById('changelog-content');

/**
 * Inicializa a janela de notas de lançamento.
 * @returns {Promise<void>}
 */
async function init() {
  try {
    // Configura o Marked para tabelas e GFM (GitHub Flavored Markdown)
    if (window.marked) {
      const options = {
        gfm: true,
        breaks: true,
        tables: true
      };
      
      if (typeof window.marked.setOptions === 'function') {
        window.marked.setOptions(options);
      }
    }

    // Busca a versão atual do app
    const version = await recorder.getAppVersion();
    if (version) {
      versionSubtitle.textContent = `Versão v${version}`;
    }

    // Busca as notas de lançamento locais (UPDATE.md)
    const body = await recorder.getReleaseNotes(version);

    if (body) {
      let processedBody = body;
      
      // Suporte simples para substituição de emojis
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
        // Converte markdown para HTML
        let html = window.marked.parse(processedBody);
        
        // Pós-processa Alertas (Admonitions) do GitHub
        const admonitionRegex = /<blockquote>\s*<p>\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\]/i;
        
        if (admonitionRegex.test(html)) {
          const tempDiv = document.createElement('div');
          tempDiv.innerHTML = html;
          
          const blockquotes = tempDiv.querySelectorAll('blockquote');
          blockquotes.forEach(bq => {
            const firstP = bq.querySelector('p');
            if (!firstP) return;
            
            const content = firstP.innerHTML.trim();
            const match = content.match(/^\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\]/i);
            
            if (match) {
              const type = match[1].toLowerCase();
              const title = match[1].toUpperCase();
              
              // Remove a tag [!TYPE] do primeiro parágrafo
              firstP.innerHTML = firstP.innerHTML.replace(/^\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\](\s*<br>)?/i, '');
              
              // Cria o container de alerta
              const alertDiv = document.createElement('div');
              alertDiv.className = `markdown-alert markdown-alert-${type}`;
              
              // Define os ícones
              let icon = '';
              switch(type) {
                case 'note':
                  icon = '<svg viewBox="0 0 16 16" width="16" height="16"><path d="M0 8a8 8 0 1 1 16 0A8 8 0 0 1 0 8Zm8-6.5a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13ZM6.5 7.75A.75.75 0 0 1 7.25 7h1a.75.75 0 0 1 .75.75v2.75h.25a.75.75 0 0 1 0 1.5h-2a.75.75 0 0 1 0-1.5h.25v-2h-.25a.75.75 0 0 1-.75-.75ZM8 6a1 1 0 1 1 0-2 1 1 0 0 1 0 2Z"></path></svg>';
                  break;
                case 'tip':
                  icon = '<svg viewBox="0 0 16 16" width="16" height="16"><path d="M8 1.5c-2.363 0-4.43 1.27-5.534 3.191-.33.576-.045 1.311.513 1.641.559.33 1.303.047 1.633-.508C5.412 4.543 6.614 3.75 8 3.75c2.347 0 4.25 1.903 4.25 4.25S10.347 12.25 8 12.25c-1.334 0-2.505-.613-3.266-1.574-.352-.445-1.015-.515-1.455-.183-.44.331-.532.964-.202 1.405L3.085 12.01A6.25 6.25 0 1 0 8 1.5ZM5 12h6a1 1 0 0 1 0 2H5a1 1 0 0 1 0-2Z"></path></svg>';
                  break;
                case 'important':
                  icon = '<svg viewBox="0 0 16 16" width="16" height="16"><path d="M0 1.75C0 .784.784 0 1.75 0h12.5C15.216 0 16 .784 16 1.75v9.5A1.75 1.75 0 0 1 14.25 13H8.06l-2.573 2.573A1.458 1.458 0 0 1 3 14.543V13H1.75A1.75 1.75 0 0 1 0 11.25Zm1.75-.25a.25.25 0 0 0-.25.25v9.5c0 .138.112.25.25.25h2a.75.75 0 0 1 .75.75v2.19l2.72-2.72a.75.75 0 0 1 .53-.22h6.5a.25.25 0 0 0 .25-.25v-9.5a.25.25 0 0 0-.25-.25Zm7 2.25v2.5a.75.75 0 0 1-1.5 0v-2.5a.75.75 0 0 1 1.5 0ZM9 9a1 1 0 1 1-2 0 1 1 0 0 1 2 0Z"></path></svg>';
                  break;
                case 'warning':
                  icon = '<svg viewBox="0 0 16 16" width="16" height="16"><path d="M6.457 1.047c.659-1.234 2.427-1.234 3.086 0l6.03 11.315c.602 1.13-.203 2.488-1.543 2.488H2.031a1.603 1.603 0 0 1-1.542-2.488ZM8 5a.75.75 0 0 0-.75.75v3.5a.75.75 0 0 0 1.5 0v-3.5A.75.75 0 0 0 8 5Zm0 9a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z"></path></svg>';
                  break;
                case 'caution':
                  icon = '<svg viewBox="0 0 16 16" width="16" height="16"><path d="M4.47.22A.75.75 0 0 1 5 0h6a.75.75 0 0 1 .53.22l4.25 4.25c.141.14.22.331.22.53v6a.75.75 0 0 1-.22.53l-4.25 4.25A.75.75 0 0 1 11 16H5a.75.75 0 0 1-.53-.22L.22 11.53A.75.75 0 0 1 0 11V5a.75.75 0 0 1 .22-.53Zm.84 1.28L1.5 5.31v5.38l3.81 3.81h5.38l3.81-3.81V5.31L10.69 1.5ZM8 4a.75.75 0 0 1 .75.75v3.5a.75.75 0 0 1-1.5 0v-3.5A.75.75 0 0 1 8 4Zm0 9a1 1 0 1 1 0-2 1 1 0 0 1 0 2Z"></path></svg>';
                  break;
              }
              
              alertDiv.innerHTML = `
                <p class="markdown-alert-title">${icon}${title}</p>
                ${bq.innerHTML}
              `;
              
              bq.parentNode.replaceChild(alertDiv, bq);
            }
          });
          
          html = tempDiv.innerHTML;
        }

        changelogContent.innerHTML = html;
        
        // Ativa o highlight do Prism
        if (window.Prism) {
          window.Prism.highlightAllUnder(changelogContent);
        }
      } else {
        changelogContent.textContent = processedBody;
      }
    } else {
      loading.innerHTML = '<p>Não foi possível carregar as notas de lançamento para esta versão.</p>';
    }
  } catch (error) {
    console.error('Erro ao carregar as notas de lançamento:', error);
    loading.innerHTML = '<p>Erro ao carregar as notas de lançamento.</p>';
  }
}

init();
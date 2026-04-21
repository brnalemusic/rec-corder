const fs = require('fs');
const https = require('https');

const diff = fs.readFileSync('pr_diff.txt', 'utf8').substring(0, 2550000);
const apiKey = process.env.GEMINI_API_KEY;

const model = "gemma-4-31b-it";
const url = `https://generativelanguage.googleapis.com/v1beta/models/${model}:generateContent?key=${apiKey}`;

// [MUDANÇA CRÍTICA]: Uso de system_instruction para isolar persona e regras
const payload = JSON.stringify({
  system_instruction: {
    parts: [{
      text: "Você é o Corder AI, o Revisor de Código oficial do projeto RecCorder. Sua função é atuar como um Revisor de Código Sênior. Sua resposta deve conter EXCLUSIVAMENTE o review em Markdown ou o emoji 👍. NUNCA mostre seu raciocínio (thoughts) ou repita instruções. Use Português do Brasil. Se o código for muito bom, maravilhoso ou excelente, responda APENAS com um emoji 👍, e absolutamente nada mais."
    }]
  },
  contents: [{
    parts: [{ text: `Analise este DIFF de Pull Request e gere o review:\n\n${diff}` }]
  }]
});

const options = {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'Content-Length': Buffer.byteLength(payload)
  }
};

const req = https.request(url, options, (res) => {
  let responseData = '';
  res.on('data', (chunk) => responseData += chunk);
  res.on('end', () => {
    if (res.statusCode !== 200) {
      console.error(`Erro na API (${res.statusCode}):`, responseData);
      fs.writeFileSync('review_result.md', `⚠️ Erro na análise da AI (Status ${res.statusCode}).`);
      process.exit(0);
    }
    try {
      const json = JSON.parse(responseData);
      const candidates = json.candidates || [];
      if (candidates.length > 0 && candidates[0].content) {
        const parts = candidates[0].content.parts;
        // Buscamos apenas a parte de texto, ignorando pensamentos/metadados
        const textPart = parts.find(p => p.text);
        const text = textPart ? textPart.text.trim() : "⚠️ Resposta vazia.";
        
        // Adiciona um cabeçalho visual se não for apenas um emoji de aprovação
        const finalContent = text === "👍" ? text : `### 🤖 RecCorder AI Review\n\n${text}`;
        fs.writeFileSync('review_result.md', finalContent);
      } else {
        fs.writeFileSync('review_result.md', "⚠️ A AI não retornou uma análise válida.");
      }
    } catch (e) {
      console.error("Erro ao processar JSON:", e);
      process.exit(1);
    }
  });
});

req.on('error', (e) => {
  console.error("Erro na requisição:", e);
  process.exit(1);
});
req.write(payload);
req.end();

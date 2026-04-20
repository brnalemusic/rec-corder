# Política de Segurança — Rec Corder

Obrigado por ajudar a manter o Rec Corder seguro. Este documento descreve como reportar vulnerabilidades, como tratamos os relatos e que boas práticas recomendamos para usuários, contribuidores e mantenedores.

## Visão geral
Rec Corder é um gravador de tela para Windows com o núcleo em Rust e uma camada de interface/instalador envolvendo JavaScript, HTML, CSS, scripts PowerShell e NSIS. Os principais vetores de risco incluem:
- confidencialidade e integridade das gravações armazenadas localmente;
- execução de código ou elevação de privilégios via instalador/scripts;
- vulnerabilidades em dependências (crates Rust, pacotes npm);
- integração entre componentes nativos e web (ex.: comunicação entre process renderer e core nativo);
- cadeia de suprimentos de build/release (builds comprometidos, instaladores não assinados).

Este arquivo orienta como reportar problemas de segurança e quais práticas seguir para reduzir riscos.

---

## Como reportar uma vulnerabilidade (canal preferencial)
Por favor, NÃO abra uma issue pública para vulnerabilidades sensíveis. Use uma das opções abaixo:

1. GitHub Security Advisory (preferível)
   - Abra um Security Advisory no repositório (GitHub → Security → Advisories) e nos marque.
2. E‑mail
   - Envie para: brenoalexandre.music@gmail.com
   - Assunto sugerido: `[SECURITY][Rec Corder] Resumo curto`
3. Relatórios altamente sensíveis
   - Se precisar criptografar a submissão, peça nossa chave PGP por e‑mail; responderemos com a chave pública.

Inclua no relatório:
- versão (tag/release/commit SHA) e informações do sistema (Windows versão, arquitetura);
- passos mínimos para reproduzir (PoC quando possível);
- impacto esperado (ex.: exfiltração de arquivos, execução remota, elevação de privilégios);
- logs, amostras, hashes dos binários/instaladores e qualquer artefato relevante;
- se envolver NSIS/PowerShell, inclua o instalador exato e flags usadas.

---

## O que fazemos após o recebimento
- Confirmamos o recebimento dentro de 7 dias úteis.
- Classificamos a gravidade e priorizamos a análise.
- Para problemas críticos, podemos trabalhar em correções em branches privados até a publicação segura do hotfix.
- Publicamos correções e releases com notas apropriadas, coordenando a divulgação com o descobridor quando aplicável.
- Prazo orientativo:
  - crítico: correção rápida, idealmente dentro de 30 dias quando factível;
  - alta/média: coordenação e correção dentro de 30–90 dias dependendo da complexidade.
- Empregamos divulgação responsável; negociaremos prazos razoáveis com quem reportou.

---

## Classificação de gravidade (exemplos aplicados ao Rec Corder)

- Crítico
  - Execução arbitrária de código com privilégios elevados através do instalador ou vulnerabilidade em componente nativo.
  - Exfiltração automática de gravações sem consentimento.
  - Corrupção irreversível de gravações do usuário.

- Alta
  - Controle remoto do aplicativo por interfaces locais/plug‑ins sem autenticação.
  - Elevação de privilégios a partir de processos que deveriam rodar sem privilégios.
  - Scripts PowerShell usados de forma maliciosa no instalador.

- Média
  - Vulnerabilidades em dependências que requerem interação do usuário para exploração.
  - Falhas que expõem caminhos de arquivo ou metadados sensíveis.

- Baixa
  - Informações úteis para um atacante presentes em logs, sem impacto direto imediato.

---

## Recomendações para usuários
- Baixe apenas releases oficiais do GitHub ou do site/documentação oficial.
- Verifique assinaturas/ hashes SHA256 dos binários/instaladores quando disponíveis.
- Evite executar instaladores com privilégios administrativos sem necessidade.
- Armazene gravações sensíveis em locais protegidos (criptografia de disco ou contêineres cifrados).
- Mantenha Windows, drivers de captura e antivírus atualizados.

---

## Recomendações para colaboradores e mantenedores
- Nunca comitar segredos no repositório; use secrets do CI.
- Habilitar e manter:
  - Dependabot/alerts para crates e pacotes JS;
  - cargo-audit / cargo-deny no CI;
  - npm audit ou ferramentas de SCA para dependências JS.
- CI e qualidade:
  - Exigir checks de CI verdes para merges (build, testes, lint, cargo-audit/npm-audit).
  - Executar `cargo fmt`, `clippy` e linter JS/TS automaticamente.
- Builds e releases:
  - Assinar código/instaladores e publicar hashes SHA256.
  - Documentar processo de build reprodutível (instruções e ambientes).
  - Automatizar builds em runner confiável e registrar proveniência quando possível.
- Scripts e instalador:
  - Revisar PowerShell e scripts NSIS para minimizar operações privilegiadas.
  - Validar e sanitizar entradas antes de executar comandos.
- Segurança do runtime:
  - Evitar exposição desnecessária de portas/IPC;
  - Isolar componentes nativos do renderer web quando aplicável (política de IPC restrita).
- Testes e fuzzing:
  - Considerar fuzzing para parsers e componentes que lidam com arquivos multimídia.
  - Adicionar testes que verifiquem limites de entrada e manipulação de arquivos.

---

## Pull requests e melhorias — boas práticas
Para manter a base segura e bem mantida, siga estas diretrizes ao abrir PRs:

- Escopo e tamanho
  - Prefira PRs pequenos e focados que resolvam um problema ou adicionem uma funcionalidade específica.
  - Se a mudança for grande, divida em PRs menores (refatoração → comportamento → testes → documentação).

- Título e descrição
  - Use um título claro e referencie issues relacionadas (`Fixes #123`).
  - Na descrição inclua:
    - objetivo da mudança,
    - impacto no usuário,
    - como testar manualmente,
    - alterações de segurança relevantes (ex.: novos privilégios solicitados, mudanças em armazenamento de gravações).

- Segurança no código
  - Declare qualquer alteração que afete permissões, acesso a arquivos, execução de comandos ou comunicação entre processos.
  - Inclua notas sobre hardening (ex.: validação de caminhos, escaping, proteções contra path traversal).
  - Não inclua credenciais ou chaves no PR.

- Testes e CI
  - Adicione/atualize testes automatizados cobrindo o novo comportamento.
  - Assegure que o CI execute: build Rust, testes unitários/integrados, lints, cargo-audit e auditoria de dependências JS.
  - Para mudanças no instalador, inclua scripts ou instruções de teste para validação manual.

- Revisão de código
  - Marque revisores relevantes (mantenedores com contexto do subsistema).
  - Solicite pelo menos uma aprovação antes do merge.
  - Requerer revisão adicional quando houver mudanças em código nativo, scripts de instalação ou código que manipule arquivos do usuário.

- Commits e histórico
  - Mantenha commits atômicos e com mensagens claras; prefira commits reescritos (squash) quando fizer sentido.
  - Use convenções de commits (ex.: Conventional Commits) se o projeto adotar.

- Documentação
  - Atualize a documentação quando houver mudanças de comportamento ou requisitos (permissões, locais de armazenamento, opções do instalador).
  - Adicione notas de segurança no changelog quando relevante.

- PRs sobre vulnerabilidades
  - Não publique PoCs sensíveis em PRs públicas. Para correções de vulnerabilidade, coordene via Security Advisory ou contato por e‑mail para evitar divulgação prematura.

---

## Política de crédito e divulgação
- Podemos creditar quem reportou a vulnerabilidade em RELEASE NOTES/ACKNOWLEDGEMENTS, salvo pedido de anonimato.
- Não há recompensa financeira automática; discussões sobre recompensas (bug bounty) serão tratadas caso a caso.

---

## Exceções legais e comportamento responsável
- Não realize testes invasivos em sistemas de usuários sem autorização.
- Solicitamos que pesquisadores sigam práticas de divulgação responsável; em caso de divulgação pública imprópria poderemos acelerar medidas para proteger usuários.

---

## Contato
- Preferível: GitHub Security Advisory do repositório.
- Alternativa: brenoalexandre.music@gmail.com (assunto: `[SECURITY][Rec Corder]`).

---

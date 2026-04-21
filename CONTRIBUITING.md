# Guia de Contribuição — Rec Corder

Obrigado por querer contribuir com o **Rec Corder**! 🎬 Queremos que você se sinta bem-vindo e confiante ao participar deste projeto.

## 💡 O Coração do Rec Corder

Antes de contribuir, entenda o que faz este projeto especial:

- **Performance é Tudo:** Cada linha de código deve considerar: "Isto vai deixar a app mais lenta?" Se sim, precisa ser otimizado.
- **Invisibilidade é o Objetivo:** O Rec Corder não deve pedir atenção — deve funcionar silenciosamente enquanto você trabalha.
- **Confiabilidade Acima de Tudo:** Usuários contam com o Rec Corder para capturar conteúdo importante. Bugs são inimigos.
- **Transparência é Confiança:** Comunique claramente o que faz, por que faz e como isso ajuda.

## 🚀 Como Começar

### 1. **Configure o Ambiente**

```bash
# Clone o repositório
git clone https://github.com/brnalemusic/rec-corder.git
cd rec-corder

# Instale as dependências (você precisa ter Rust instalado)
# Acesse: https://www.rust-lang.org/tools/install

# Instale as dependências do projeto
cd src-tauri; cargo build

# Volte para a pasta padrão e rode o debug
cd ..; npm run tauri dev
```
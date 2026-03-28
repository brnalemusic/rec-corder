## 🎬 O que é o Rec Corder?

O **Rec Corder** foi criado para ser o gravador de tela mais rápido e eficiente que você já usou. 
Enquanto outros apps pesam centenas de MBs e consomem toda a sua CPU, o Rec Corder utiliza o poder do seu hardware (placa de vídeo) para processar vídeos em **alta qualidade (até 60 FPS)** com impacto quase zero no seu PC.

> [!IMPORTANT]
> Esta é uma versão **Pre-Alpha**. O processamento de áudio + vídeo ocorre de forma assíncrona ao final da gravação. Aguarde a mensagem de "Salvo" antes de fechar o app!

---

## 📦 Como Baixar e Usar (Sem Estresse)

Não precisa saber programar para usar! 

1. Acesse a aba [**Releases**](https://github.com/brnalemusic/rec-corder/releases) (lado direito deste repositório).
2. Baixe o arquivo `Rec Corder_0.1.2_x64-setup.exe`.
3. Execute o instalador.
4. Se o Windows mostrar o aviso do SmartScreen, clique em **"Mais informações"** e depois em **"Executar assim mesmo"**.

---

## ✨ Principais Recursos

- 🚀 **Hardware Accelerated:** Suporte nativo para **NVENC (NVIDIA)**, **AMF (AMD)** e **QuickSync (Intel)**.
- 🎤 **Áudio Dual:** Grave o som do seu sistema e do seu microfone simultaneamente.
- 📐 **Custom Scale:** Reduza a escala do vídeo (80%, 60%) para poupar gigabytes de espaço.
- ⚡ **Zero Lag:** Interface escrita em Vanilla JS e Backend em Rust para latência mínima.
- 🛠️ **Recuperação de Falhas:** Se o seu PC desligar, o app tenta recuperar a gravação na próxima inicialização.

---

## 🛠️ Para Desenvolvedores (Build Manual)

Se você quer modificar o código ou contribuir:

### Pré-requisitos
- [Node.js](https://nodejs.org/) (v18+)
- [Rust & Cargo](https://rustup.rs/)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

### Como rodar:
```bash
# Instale as dependências
npm install

# Inicie o app em modo de desenvolvimento
npm run tauri dev

# Gere o seu próprio instalador de produção
npm run tauri build
```

---

## 🤝 Contribuições

Este projeto é **100% open-source**. Sinta-se livre em abrir Issues para relatar bugs ou Pull Requests com melhorias de código!

---

<div align="center">
  <sub>Criado com 🧡 para a comunidade criativa.</sub>
</div>

# 🎬 Rec Corder — Gravador de Tela para Windows e Linux

O **Rec Corder** foi desenvolvido para ser o gravador de tela mais "invisível" que você já usou. 

Enquanto gravadores tradicionais consomem sua CPU e podem causar engasgos (stuttering) durante o jogo ou trabalho, o Rec Corder utiliza o poder bruto da sua placa de vídeo (Hardware Acceleration) e uma arquitetura híbrida inteligente para garantir que o desempenho do seu PC permaneça intocado enquanto você captura tudo o que acontece na tela.

---

### ✨ O que o torna especial?

- **Prioridade ao Usuário:** O Rec Corder é "egoísta" com o hardware — no bom sentido. Ele consome o mínimo de CPU e RAM durante a gravação para que seu jogo ou software pesado rode liso.
- **Processamento Transparente:** Quando você clica em "Parar", o app entra no estado de **Processando...**. É nesse momento que ele combina o vídeo capturado com o áudio cristalino. Esse pequeno intervalo ao final é o "segredo" por trás da performance extrema durante a gravação.
- **Zero Lag Real:** Nada de interfaces pesadas. Tudo aqui foi escrito em Vanilla JS e Rust (Tauri) para abrir instantaneamente e responder ao seu clique sem pensar duas vezes.
- **Segurança de Dados:** Se o seu PC desligar ou o Windows travar, o Rec Corder possui um **Watchdog (Cão de Guarda)** que tenta recuperar sua gravação na próxima vez que o app for aberto.

---

### 🚀 Principais Recursos

- 🎮 **Aceleração por Hardware:** Suporte nativo para **NVENC (NVIDIA)**, **AMF (AMD)**, **QuickSync (Intel)** e **VAAPI (Linux)**.
- 🔊 **Áudio de Alta Fidelidade:** Captura nativa via Windows WASAPI ou Linux PulseAudio, garantindo o som do sistema e microfone sem latência.
- 📐 **Escala Customizável:** Reduza a imagem (80%, 60%) para poupar gigabytes de espaço mantendo a fluidez.
- 📺 **60 FPS Estável:** Gravações fluidas ideais para gameplay e tutoriais profissionais.
- 📁 **Pasta de Destino Rápida:** Mude o local de salvamento em um clique direto na interface.

---

### 📦 Como Instalar (Sem Estresse)

Não precisa saber programar para usar! Siga estes passos simples:

1. Acesse a aba [**Releases**](https://github.com/brnalemusic/rec-corder/releases) (lado direito deste repositório).
2. Baixe a versão para o seu sistema:
   - **Windows:** Baixe o arquivo `.exe`.
   - **Linux:** Baixe o arquivo `.deb` (Debian/Ubuntu).
3. Execute o instalador. Ele configurará automaticamente todos os componentes necessários (incluindo o FFmpeg embutido).
4. No Windows: Se mostrar o aviso do SmartScreen, clique em **"Mais informações"** e depois em **"Executar assim mesmo"**.

---

### 🛠️ Especificações Técnicas (Para Curiosos)

O Rec Corder é construído com tecnologia de ponta:
- **Core:** [Rust](https://www.rust-lang.org/) + [Tauri v2](https://v2.tauri.app/) (Segurança e Baixo Consumo).
- **VÍDEO:** 
  - **Windows:** [Windows Graphics Capture (WGC)](https://learn.microsoft.com/en-us/windows/uwp/audio-video-camera/screen-capture) via FFmpeg.
  - **Linux:** X11 Grab via FFmpeg.
- **Áudio:** 
  - **Windows:** WASAPI PCM.
  - **Linux:** PulseAudio PCM.
- **Frontend:** Vanilla JS e Modern CSS (Sem frameworks pesados).

---

### 🤝 Contribuições

Este projeto é **100% open-source**. Se você encontrar um bug ou tiver uma ideia de recurso, abra uma **Issue** ou envie um **Pull Request**.

<div align="center">
  <sub>Criado com 🧡 para a comunidade criativa. v1.0.0-beta.5</sub>
</div>

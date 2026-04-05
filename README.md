# 🎬 Rec Corder — Gravador de Tela Ultra-Otimizado e de Alta Performance para Windows

O **Rec Corder** foi desenvolvido para ser o gravador de tela mais "invisível" que você já usou. 

Enquanto gravadores tradicionais consomem sua CPU e podem causar engasgos (stuttering) durante o jogo ou trabalho, o Rec Corder utiliza o poder bruto da sua placa de vídeo (Hardware Acceleration) e uma arquitetura híbrida inteligente para garantir que o desempenho do seu PC permaneça intocado enquanto você captura tudo o que acontece na tela.

---

### ✨ O que o torna especial? (Lado Humano)

- **Prioridade ao Usuário:** O Rec Corder é "egoísta" com o hardware — no bom sentido. Ele consome o mínimo de CPU e RAM durante a gravação para que seu jogo ou software pesado rode liso.
- **Processamento Transparente:** Quando você clica em "Parar", o app entra no estado de **Processando...**. É nesse momento que ele combina o vídeo capturado com o áudio cristalino. Esse pequeno intervalo ao final é o "segredo" por trás da performance extrema durante a gravação.
- **Zero Lag Real:** Nada de interfaces pesadas. Tudo aqui foi escrito em Vanilla JS e Rust (Tauri) para abrir instantaneamente e responder ao seu clique sem pensar duas vezes.
- **Segurança de Dados:** Se o seu PC desligar ou o Windows travar, o Rec Corder possui um **Watchdog (Cão de Guarda)** que tenta recuperar sua gravação na próxima vez que o app for aberto.

---

### 🚀 Principais Recursos

- 🎮 **Aceleração por Hardware:** Suporte nativo para **NVENC (NVIDIA)**, **AMF (AMD)** e **QuickSync (Intel)**.
- 🔊 **Áudio de Alta Fidelidade:** Captura nativa via Windows WASAPI, garantindo o som do sistema e microfone sem latência.
- 📐 **Escala Customizável:** Reduza a imagem (80%, 60%) para poupar gigabytes de espaço mantendo a fluidez.
- 📺 **60 FPS Estável:** Gravações fluidas ideais para gameplay e tutoriais profissionais.
- 📁 **Pasta de Destino Rápida:** Mude o local de salvamento em um clique direto na interface.

---

### 📦 Como Instalar (Sem Estresse)

Não precisa saber programar para usar! Siga estes passos simples:

1. Acesse a aba [**Releases**](https://github.com/brnalemusic/rec-corder/releases) (lado direito deste repositório).
2. Baixe o arquivo `.exe` mais recente.
3. Execute o instalador. Ele configurará automaticamente todos os componentes necessários para você.
4. Se o Windows mostrar o aviso do SmartScreen, clique em **"Mais informações"** e depois em **"Executar assim mesmo"**.

---

### 🛠️ Especificações Técnicas (Para Curiosos)

O Rec Corder é construído com tecnologia de ponta para Windows:
- **Core:** [Rust](https://www.rust-lang.org/) + [Tauri v2](https://v2.tauri.app/) (Segurança e Baixo Consumo).
- **Vídeo:** [Windows Graphics Capture (WGC)](https://learn.microsoft.com/en-us/windows/uwp/audio-video-camera/screen-capture) via FFmpeg para captura com zero cópia de memória.
- **Áudio:** Captura nativa PCM via WASAPI com mixagem assíncrona.
- **Frontend:** Vanilla JS e Modern CSS (Sem frameworks pesados).

---

### 🤝 Contribuições

Este projeto é **100% open-source**. Se você encontrar um bug ou tiver uma ideia de recurso, abra uma **Issue** ou envie um **Pull Request**.

<div align="center">
  <sub>Criado com 🧡 para a comunidade criativa. v0.2.6</sub>
</div>


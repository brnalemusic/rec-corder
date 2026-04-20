Esta é uma versão beta 4 para desenvolvimento. Recomendamos utilizar a versão estável, sendo, atualmente, a ``v0.4.0``.

> [!WARNING]
> A aplicação atual pode vir com bugs.
> A melhor experiência pode ser encontrada utilizando a versão estável.
> Espere por muitos bugs e problemas de estabilidade.
> Atenciosamente, Breno

# Rec Corder v1.0.0-beta.4
O Rec Corder versão 1 recebeu algumas atualizações na sua beta 4. Uma dessas atualizações foi a adição de uma nova plataforma CLI que pode ser instalada junto com o programa. O CLI foi construído em Python e utiliza o PyO3 para interagir com o Rust da aplicação. O CLI foi devidamente testado com o `npm run cli:dev` (ambiente de debug), mas não foi buildado localmente antes do lançamento.

### Testando o ambiente CLI
Para testar o ambiente CLI, existem duas maneiras. A primeira, é no [Modo Desenvolvedor](#Desenvolvedor). A segunda, é instalando a aplicação e aceitando o Download do CLI via Instalador. Um Pop-up aparecerá, e você deverá clicar em "Sim" para instalar o CLI. Esse é o método para [Usuários Finais](#Usuários).

# Desenvolvedor
Para testar sendo um desenvolvedor, você deve clonar esse repositório na sua máquina, buildar a aplicação e rodar o comando de CLI no modo Debug. Veja abaixo como fazer:

``````Bash
# Clone o repositório
git clone https://github.com/brnalemusic/rec-corder.git

# Entre no diretório onde contém o Cargo.toml
cd rec-corder\src-tauri

# Faça o build (pode demorar de 20 a 40 minutos)
cargo check
cd ..
npm run build:release

# Agora, rode o CLI via Debug
npm run cli:dev
``````

# Usuários
Para testar sendo um usuário, a única coisa que você precisa fazer é o Download da aplicação. Vá até a parte de baixo desta Release e baixe o arquivo para a sua OS.

|Sistema Operacional|Suporte|Tipo de Arquivo|
|---|---|---|
|Windows|✅|`.exe`|
|Linux (Ubuntu e Debian)|✅|`.deb` ou `.AppImage`|
|MacOS|❌|Não há suporte para MacOS|

Após finalizar o download do instalador, execute-o. No final, ele pedirá permissão para instalar o CLI. Aceite.

Pronto! Agora é só rodar `reccorder` no seu terminal e utilizar o Rec Corder versão CLI, feito em Python e Rust.

### O que há de novo na v1.0.0-beta.4:
- **Suporte CLI Cross-Platform:** O CLI agora é buildado e testado automaticamente para Windows e Linux.
- **Otimização no Linux:** Melhoria drástica na velocidade de detecção de dispositivos de áudio e vídeo em sistemas Linux/PipeWire.
- **Ambiente Python Isolado:** No Windows, o CLI utiliza um ambiente Python 3.11 embutido e isolado.
- **Correções na API PyO3:** Melhor tratamento de erros ao inicializar sessões de gravação via linha de comando.

> [!TIP]
> O Rec Corder CLI agora é totalmente funcional tanto no Windows quanto no Linux (via Python do sistema).

> [!IMPORTANT]
> Esta versão foca na estabilidade do Workflow de Release e na integração CLI.
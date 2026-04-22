# Rec Corder v1.0.0
**Finalmente!** Eu disse: **Finalmente!** A versão estável de Rec Corder para Linux e Windows está no ar.

De `v0.4.0` —> `v1.0.0`
Por [Breno Alexandrē](https://brnalemusic.vercel.app)

## O que isso muda pra você?
Muita coisa! Nossa aplicação passou por mudanças radicais no ecossistema de gravação, adicionamos suporte para download do CLI via Windows, e o mais importante: o Rec Corder agora roda em Linux! Sim, para usuários de Linux, agora o Rec Corder é totalmente possível.

#### Ressalvas
O Rec Corder não é completamente compatível com Hyprland, e possui alguns bugs relacionados. É de nossa responsabilidade fazer as alterações e atualizações necessárias de correções de bugs para fazer funcionar. Espere por mais updates!

# O que foi implementado
O Rec Corder sofreu alguns updates durante sua fase beta que pode ser extremamente sustentável pra aplicação futuramente!

1. **AI Review para o GitHub:** O repositório público oficial (brnalemusic/rec-corder) agora consta com um assistente de IA para Pull Requests, Commits, Issues, etc. Como o repositório ainda é muito novo, não há colaboradores humanos para me ajudarem com Pull Requests e Issues.

2. **Suporte para Linux:** O grande update é o suporte nativo à Linux, especificamente Ubuntu e/ou Debian.

3. **Melhor suporte à tela cheia:** O Rec Corder recebeu um suporte mais rígido à aplicações em Tela Cheia.

E muito mais.

O diff completo pode ser encontrado em: https://github.com/brnalemusic/rec-corder/compare/v0.4.0...v1.0.0

## Download
É simples: basta [clicar aqui](https://www.reccorder.com.br) para baixar a versão mais atualizada do aplicativo através do website oficial. Se você estiver pelo aplicativo, basta clicar em "Atualizar e Reiniciar" para atualizar de forma interna, e caso você esteja vendo isso tardiamente, com outras versões disponíveis após essa, e mesmo assim queira baixar esta versão, basta clicar no instalador `.exe` abaixo (não recomendado).

### Para Devs
O download para devs exige alguns comandos no seu terminal. Execute os seguintes comandos, em ordem:
``````ps
# Clona o repositório para a sua máquina
git clone "https://github.com/brnalemusic/rec-corder.git"

# Entra na pasta do repositório
cd "rec-corder"

# Instala as dependências
npm install

# Roda o aplicativo diretamente na sua máquina
npm run tauri dev
``````
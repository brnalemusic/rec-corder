# Rec Corder v0.4.0
Estamos muito contentes em lançar a versão 0.4.0 do Rec Corder. Agora, o aplicativo conta com funcionalidades essenciais que a versão anterior não tinha.

**De:** `v0.3.1` —> `v0.4.0`
**Por** [Breno Alexandrē](https://brnalemusic.vercel.app)

## Principais updates
O update principal da aplicação é o seu suporte à aplicações em tela cheia. Em versões anteriores, não era possível gravar aplicativos em tela cheia, já que ocorria o famoso *congelamento por tela duplicada*. Isso foi corrigido, e agora nossa aplicação utiliza de configurações mais razoáveis para gravações em tela cheia e/ou desktop.

# Resumo das Atualizações
|Atualização|`v0.3.1`|`v0.4.0`|Comentário|
|--:|-:-|-:-|:--|
|Tela Cheia|❌|✅|A aplicação agora suporta gravações em tela cheia e/ou desktop.|
|Webcam|✅|✅|A aplicação teve otimizações na gravação de webcam.|
|Desempenho|✅|✅|A aplicação teve otimizações de desempenho.|

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

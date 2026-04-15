# Rec Corder v0.3.1
Corrigimos um bug que não mostrava as Release Notes na janela dedicada (versão Build).

**De:** `v0.3.0` —> `v0.3.1`
**Por** [Breno Alexandrē](https://brnalemusic.vercel.app)

## Principais updates
Essa versão trás apenas uma atualização, mas que é muito importante para a experiência do usuário. Agora, você poderá visualizar as notas de lançamento da versão que você está utilizando diretamente pelo aplicativo. Um bug no Build da versão anterior ocorria e impedia que as notas fossem exibidas, mas agora, o problema foi corrigido.

#### Updates
1. **Notas** — Ao clicar no número da versão, você consegue ver as notas de lançamento da versão que você está utilizando.

### Diff das atualizações
|Recurso|``v0.3.0``|``v0.3.1``|Comentários|
|--:|:-:|:-:|:--|
|Notas|✅|✅|O sistema antigo foi consertado.|

#### Detalhes
1. **Notas**
Ao clicar na versão do aplicativo, no canto superior direito, a aplicação exibe as notas da atualização que você está utilizando agora. Ele lê o arquivo ``UPDATE.md`` disponível no Source Code da aplicação (no momento do build do GitHub Actions).

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

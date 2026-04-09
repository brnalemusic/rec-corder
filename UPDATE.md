# Rec Corder v0.3.0
Estamos trabalhando duro para adicionar funcionalidades incríveis à nossa aplicação de código aberto. Esse é um MINOR update que trará uma funcionalidade incrível à aplicação: a possibilidade de utilizar a Webcam nas gravações!

E, finalmente, saimos dos updates PATCH para arquitetar algo que será realmente útil, sem afetar o desempenho da gravação.

**De:** `v0.2.12` —> `v0.3.0`
**Por** [Breno Alexandrē](https://brnalemusic.vercel.app)

## Novidades
Estamos trabalhando para adicionar funcionalidades realmente pros criadores e pros usuários. Dessa vez, implementamos as funcionalidades mais comuns de mercado para o melhor desempenho possível.

### Arquivos modificados
Os seguintes arquivos foram modificados, adicionados ou removidos:

|Arquivo|Alteração|Comentário|
|--:|:-:|:--|
|``config.rs``|Modificado|Adicionado 4 novos campos à struct ``AppConfig``|
|``recorder.rs``|Novo|Novo ``#[tauri::command]`` que lista câmeras via FFmpeg.|
|``mod.rs``|Modificado|Sem mudança de exportação necessária — ``recorder.rs`` já é re-exportado.|
|``lib.rs``|Modificado|Adicionado ``recorder::list_cameras`` ao ``invoke_handler``.|
|``ffmpeg.rs``|Novo|Nova função pública para montar o input e filtro de overlay da webcam.|
|``session.rs``|Novo|Diversas alterações.|

Diversas outras alterações foram feitas no código, mas essas foram as maiores. Caso queira ver o relatório completo, você pode [comparar as versões](https://github.com/brnalemusic/rec-corder/compare/v0.2.11...v0.3.0) e ter todo o log de alterações de arquivos ultra detalhado.

## Principais updates
Note que essa versão não trás apenas a Webcam para a lista de updates. Nós fizemos diversas melhorias que irá transformar sua experiência com o Rec Corder. Abaixo, veja uma lista de todos os updates que foram implementados para a versão ``v0.3.0``.

#### Updates
1. **Webcam** — Agora você consegue utilizar a Webcam como sobreposição nas suas gravações.

2. **Notas** — Ao clicar no número da versão, você consegue ver as notas de lançamento da versão que você está utilizando.

3. **Antiacidente** — Ao clicar no X, você é alertado sobre o fechamento e o cancelamento da gravação, sem meios de recuperar. Implementaremos *Smart Cancel* em atualizações futuras.

### Diff das atualizações
|Recurso|``v0.2.11``|``v0.3.0``|Comentários|
|--:|:-:|:-:|:--|
|Webcam|❌|✅|-|
|Notas|❌|✅|-|
|Antiacidente|✅|✅|O sistema foi aprimorado.|

#### Detalhes
1. **Webcam**
O recurso de Webcam foi adicionado através de uma comunicação interna entre o Frontend e o Backend, que carrega o recurso junto com a sua tela através do FFmpeg. No final, a Webcam é achatada em seu vídeo, tornando-o em um vídeo único. Você pode escolher o posicionamento e o tamanho da sua câmera através das configurações > vídeo.

2. **Notas**
Ao clicar na versão do aplicativo, no canto superior direito, a aplicação exibe as notas da atualização que você está utilizando agora. Ele lê o arquivo ``UPDATE.md`` disponível no Source Code da aplicação (no momento do build do GitHub Actions).

3. **Antiacidente**
O aplicativo pede a sua confirmação para fechar a aplicação quando você clica no X nativo do Windows. Um modal estilizado é aberto na frente da aplicação pedindo a confirmação do usuário para efetuar o fechamento. O usuário receberá um alerta dizendo que a gravação ficará corompida.

> [!IMPORTANT]
> O sistema antiacidente agora informa ao usuário que ele clicou em fechar antes de realmente abortar a execução da aplicação, resultando em corrupção do vídeo.
> Essa foi uma atualização sugerida por testers da aplicação. Um grande abraço à Dave Santos pelo feedback.

> [!TIP]
> Teste a sua câmera! Vá nas configurações e altere a câmera, posicionamento e tamanho.
> Essa funcionalidade pode bugar e ainda está sendo melhorada. Caso encontre alguma falha, abra uma Issue.

## Próximas atualizações
Para updates futuros, estamos planejando alterações que adicionarão ainda mais profissionalismo na nossa plataforma. A filosofia do Rec Corder é simples: otimização e fluidez. E é isso que iremos cumprir.

### O que será adicionado futuramente?
1. **Antiacidente Nuclear**
Para fechamentos acidentais da aplicação, sem incluir o desligamento repentino (como tela azul, queda de energia), iremos adicionar uma função que salva a gravação antes de realmente fechar. Isso impede da gravação corromper. Chamaremos esse recurso de *Smart Cancel*.

2. **Melhorias no Desempenho**
Estamos planejando fazer melhorias no desempenho da aplicação para torná-la mais estável e otimizada. Estamos cientes que essa atualização está destilando o desempenho original das versões anteriores, e isso vem desde a versão ``v0.2.0`` por conta da adição do WGC.

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

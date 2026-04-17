Esta é uma versão beta 2 para desenvolvimento. Recomendamos utilizar a versão estável, sendo, atualmente, a ``v0.3.1``.

> [!WARNING]
> A aplicação atual pode vir com bugs.
> A melhor experiência pode ser encontrada utilizando a versão estável.
> Espere por muitos bugs e problemas de estabilidade.
> Atenciosamente, Breno

# Rec Corder v1.0.0-beta.2
O Rec Corder versão 1 recebeu algumas atualizações na sua beta, que melhoram compatibilidade com o Snap e fazem os arquivos `.deb` e `.appimage` finalmente funcionarem corretamente como deveriam.

## O que foi atualizado?
**Novo arquivo** *`src-tauri/com.reccorder.app.metainfo.xml`* foi adicionado ao repositório, dando as informações corretas do Metadata da aplicação para o Ubuntu App Center ao rodar o `.deb`. Agora, também é possível instalar via comando `sudo apt install ./app.deb` caso você tenha o repositório local (para devs).

**Workflow Corrigido** em *`.github/workflows/release.yml`* para que o GitHub utilize o binário do repositório ao invés de uma página GitHub que não existe para download offline do FFmpeg.

# Download
O download dessa versão não é recomendado.
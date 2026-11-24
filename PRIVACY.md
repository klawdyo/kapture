# Política de Privacidade

O **kapture** é um gravador de tela local para Windows. Ele foi feito para funcionar sem coletar dados pessoais.

## Dados coletados

O kapture **não coleta, não armazena remotamente e não envia** nenhum dado pessoal, telemetria, estatística de uso, gravação de tela, áudio do microfone, imagem da webcam ou relatório de erro.

Tudo o que você grava fica exclusivamente na pasta escolhida no seu computador.

## Logs de diagnóstico

Para ajudar a investigar problemas técnicos, o kapture registra eventos operacionais localmente em `%LOCALAPPDATA%\kapture\logs\kapture.log` (no máximo 3 arquivos de 512 KB com rotação automática). Esses arquivos nunca são enviados pela internet: eles só saem do seu computador se você decidir compartilhá-los manualmente.

## Acesso à internet

Ao abrir o aplicativo, o kapture faz apenas uma consulta HTTPS comum ao GitHub (`github.com/klawdyo/kapture/releases/latest/download/` ou `api.github.com/repos/klawdyo/kapture/releases/latest`) para verificar se existe uma versão mais recente disponível.

Nenhum identificador pessoal ou informação do seu sistema além dos cabeçalhos padrão de uma requisição HTTPS é enviado. Se quiser impedir até essa consulta, basta bloquear a saída de rede do executável `cap.exe` no Firewall do Windows.

## Contato

Dúvidas sobre esta política podem ser abertas em [github.com/klawdyo/kapture/issues](https://github.com/klawdyo/kapture/issues).

---
name: ssh-no-windows
description: Chave SSH que o servidor OpenSSH do Windows recebe e recusa, mesmo com o arquivo certo no lugar. Cobre o authorized_keys do usuário contra o administrators_authorized_keys, a ACL que o serviço (SYSTEM) precisa para ler, o grupo "Administradores" em Windows pt-BR que não casa com o "administrators" do sshd_config, e o sshd -T que decide qual arquivo o servidor realmente lê. Carregar ao configurar SSH por chave para uma VM Windows, ao depurar "Permission denied (publickey)" com a chave visivelmente certa, e antes de escrever authorized_keys por PowerShell.
---

# Chave SSH no Windows: o arquivo certo e a ACL que o serviço lê

Caso de origem (2026-11-21): o Mac oferecia a chave, o servidor `OpenSSH_for_Windows_9.5`
a **recebia** e recusava — a autenticação continuava em `publickey,password,keyboard-interactive`.
O `authorized_keys` do usuário estava no lugar, com o conteúdo certo (109 bytes), mas a ACL
tinha ficado **só com o dono**. O `sshd` roda como `SYSTEM` e não conseguia ler o arquivo.
O log `[preauth]` sem `bad ownership or modes` é consistente com arquivo ilegível, não com chave errada.

## O que decide: `sshd -T`, não a suposição

O `sshd_config` do Windows tem um bloco `Match Group administrators` que troca o
`AuthorizedKeysFile` para `__PROGRAMDATA__/ssh/administrators_authorized_keys`. Em Windows
**pt-BR** o grupo é `Administradores`, e o nome em inglês do bloco **não casa** — mas isso
precisa ser confirmado, não presumido:

```powershell
& "$env:WINDIR\System32\OpenSSH\sshd.exe" -T -C user=Jeferson,host=localhost,addr=127.0.0.1 |
  Select-String 'authorizedkeysfile|pubkeyauthentication|passwordauthentication'
```

O que vier em `authorizedkeysfile` é o arquivo que o serviço lê. Se for
`__PROGRAMDATA__/ssh/administrators_authorized_keys`, o arquivo do usuário é ignorado;
se for `.ssh/authorized_keys`, o do admin é que sobra.

## Três armadilhas medidas

1. **`icacls` com nome de grupo falha por idioma.** `net localgroup administrators` não
   acha o grupo em pt-BR (é `Administradores`); o teste correto é por SID `S-1-5-32-544`.
2. **`FileSystemAccessRule("S-1-5-18", ...)` é inválido.** O construtor espera **nome de
   conta**; para SID é preciso `New-Object Security.Principal.SecurityIdentifier('S-1-5-18')`.
   O erro é `IdentityNotMappedException`.
3. **`>>`/`Out-File` no PowerShell 5.1 grava UTF-16.** O `authorized_keys` precisa ser
   UTF-8 **sem BOM**; use `[IO.File]::WriteAllText($path, $chave + "`n", [Text.UTF8Encoding]::new($false))`.

## A correção que funciona

O serviço precisa de ACE para `SYSTEM` (S-1-5-18) e para o grupo de administradores
(S-1-5-32-544), e a herança deve ser removida para não voltar a depender do dono:

```powershell
$key  = 'ssh-ed25519 AAAA... usuario@host'
$path = 'C:\ProgramData\ssh\administrators_authorized_keys'
[IO.File]::WriteAllText($path, $key + "`n", [Text.UTF8Encoding]::new($false))
icacls $path /inheritance:r /grant:r "*S-1-5-18:(F)" "*S-1-5-32-544:(F)"
Restart-Service sshd
```

Para o arquivo do usuário (`~/.ssh/authorized_keys`), a mesma lógica: `SYSTEM` precisa
ler, e a herança não pode depender só do dono. O sintoma de ACL errada é o servidor
receber a chave e recusar sem dizer por quê.

## Como saber que funcionou

Do lado do cliente, `ssh -vvv` mostra `Server accepts key` e depois
`Authenticated to ... using "publickey"`. Do lado do servidor, o log `OpenSSH/Operational`
registra `Accepted publickey for <user>`. Se o cliente diz "Server accepts key" e o
servidor recusa, o problema é **leitura do arquivo** (ACL/caminho), não a chave.

## Higiene da VM depois do teste

O OBS (ou qualquer app de desktop) lançado por SSH **morre quando a sessão fecha** — o
processo fica preso ao job object da sessão. Para manter vivo, a sessão SSH precisa
continuar aberta. E um `.sentinel` de crash anterior em
`%APPDATA%\obs-studio\.sentinel` faz o OBS abrir no diálogo de recuperação e travar
antes de gravar log; limpar o diretório e subir com `--disable-shutdown-check` resolve.

# Runbook — assinar o release do Android

## Onde estão as coisas (fora do repositório)

| Item | Caminho | Nota |
|---|---|---|
| Keystore de upload | `~/.kentron/kentron-upload.jks` | PKCS12, alias `kentron`, validade até **2054-04-09**, permissão `600` |
| Senha | `~/.kentron/keystore.pass` | arquivo `600`; **nunca** impressa em log, commit ou terminal compartilhado |
| SHA-1 | `B5:B4:4B:D5:16:B4:C0:0D:1D:0B:81:63:CE:58:F3:57:6F:01:C8:9A` | para restrição de chave de API e Firebase |
| SHA-256 | `51:90:1C:56:9A:1B:F4:F9:03:53:82:17:21:12:66:3A:44:FF:6C:7C:9D:6F:E9:71:B4:68:0F:EF:53:03:5D:5C` | idem |

Gerado em 2026-11-22 com `keytool -genkeypair` (RSA 2048, `-validity 10000`). **Faça backup** dos dois arquivos.

## Regras de não vazamento

- `.gitignore` cobre `*.jks`, `*.keystore`, `key.properties`, `.env`, `.env.*` (com exceção de `.env.example`) e `.envrc`.
- **`key.properties` não é usado neste projeto.** Quando a assinatura for ligada, o Gradle lê de **variáveis de ambiente** (ou de `~/.kentron`), para o segredo não morar na árvore do repositório.
- O gate `gitleaks` roda no `.githooks/pre-push` e reprova padrão de credencial.
- **Antes de qualquer commit**, as duas provas abaixo têm de vir **vazias**:

```bash
git ls-files | grep -E '(^|/)(\.env|.*\.jks|.*\.keystore|key\.properties)$'
git grep -I -E 'AIza[0-9A-Za-z_-]{35}'
```

## Play App Signing

A chave criada aqui é a **upload key**: ela assina o pacote que sobe para a Play, e o Google assina o que vai para o usuário. Consequência prática: **perder a upload key não é fatal** (dá para pedir substituição), e o SHA-1 que o app publicado apresenta é o **da Play**, não o desta chave. Por isso, ao restringir uma chave de API por Android, **as duas** impressões entram: a de upload (para build local) e a da Play (para o app da loja).

## Quando for assinar de verdade

Ainda **não há configuração de assinatura** no `apps/kentron_app/android/app/build.gradle.kts`. Quando houver release para publicar, o passo é: ler as variáveis de ambiente no Gradle, falhar alto se faltarem, e gerar o `bundleRelease`. Este runbook é atualizado nesse momento — não antes, para não descrever um procedimento que não existe.
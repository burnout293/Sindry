<#
.SYNOPSIS
    Construit l'archive de la version portable de Sindry (2.4.0).

.DESCRIPTION
    A partir des binaires deja compiles par `tauri build` :

      Sindry_<version>_x64-portable.zip
        Sindry\
          Sindry.exe        l'application (sindry.exe)
          sindry.com        la ligne de commande (sindry-cli.exe)
          categories.json   les ressources que l'installeur pose a cote de l'exe
          lots.json
          sindry-portable   le marqueur : ce dossier est une version portable
          LISEZ-MOI.txt

    Le marqueur suffit : au lancement, Sindry range ses donnees dans
    `Donnees\` a cote de lui (src-tauri/src/portable.rs).

    Avec -ReleaseId, l'archive est jointe a la release (le brouillon que
    tauri-action vient de creer). Son empreinte SHA-256 sort en annotation et
    rejoint empreintes.txt, comme celle de l'installeur.

.EXAMPLE
    ./tools/construire-portable.ps1 -Version v2.4.0
    ./tools/construire-portable.ps1 -Version v2.4.0 -ReleaseId 123456
#>
param(
    [Parameter(Mandatory)] [string] $Version,
    [string] $ReleaseId,
    [string] $Sortie = 'portable'
)
$ErrorActionPreference = 'Stop'

# ==============================================================================
# Variables
# ==============================================================================

# Racine du depot, a cote de ce script.
$Racine = Split-Path -Parent $PSScriptRoot
# Les binaires compiles par tauri build.
$Binaires = Join-Path $Racine 'src-tauri/target/release'
# Le numero sans le « v » du tag.
$Numero = $Version.TrimStart('v')
# Le nom de l'archive, sans extension.
$Nom = "Sindry_${Numero}_x64-portable"
# Le dossier que l'archive contiendra.
$Dossier = Join-Path $Sortie "$Nom/Sindry"
# Ce que l'utilisateur lit en ouvrant l'archive.
$LisezMoi = @"
Sindry $Numero - version portable

- Lancez Sindry.exe. Rien ne s'installe sur le PC : reglages, catalogues,
  scripts et journaux vont dans le dossier Donnees, a cote de lui.
- Ce qui protege le PC (approbations des scripts, emplacements proteges,
  catalogues ajoutes) reste dans le PC, et ne se modifie qu'en administrateur.
- Mise a jour : telechargez la nouvelle archive, et remplacez les fichiers de
  ce dossier par les siens. Gardez le dossier Donnees : vos reglages y sont.
- Si une action redemarre le PC, rouvrez Sindry depuis la cle apres le
  redemarrage : il proposera de reprendre.
- « Windows a protege votre PC » : Sindry n'est pas signe numeriquement.
  Cliquez sur « Informations complementaires », puis « Executer quand meme ».
- Windows 10 ou 11, 64 bits, avec WebView2 (present sur Windows 11, et sur
  Windows 10 a jour).

https://burnout293.github.io/Sindry/
"@
# Le marqueur dit lui-meme a quoi il sert.
$Marqueur = "Ce fichier fait de ce dossier une version portable de Sindry : ses donnees vont dans Donnees\, a cote. Sans lui, Sindry utiliserait celles du PC."

# ==============================================================================
# Execution
# ==============================================================================

Remove-Item -Recurse -Force (Join-Path $Sortie $Nom) -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $Dossier | Out-Null

foreach ($f in @(@('sindry.exe', 'Sindry.exe'), @('sindry-cli.exe', 'sindry.com'))) {
    $source = Join-Path $Binaires $f[0]
    if (-not (Test-Path -LiteralPath $source)) {
        Write-Host "::error title=Version portable::$($f[0]) introuvable dans $Binaires : construisez d'abord (tauri build)."
        exit 1
    }
    Copy-Item -LiteralPath $source -Destination (Join-Path $Dossier $f[1])
}
Copy-Item -LiteralPath (Join-Path $Racine 'tools/categories.json') -Destination $Dossier
Copy-Item -LiteralPath (Join-Path $Racine 'tools/lots.json') -Destination $Dossier
[IO.File]::WriteAllText((Join-Path $Dossier 'sindry-portable'), $Marqueur, [Text.UTF8Encoding]::new($false))
[IO.File]::WriteAllText((Join-Path $Dossier 'LISEZ-MOI.txt'), $LisezMoi.Replace("`r`n", "`n").Replace("`n", "`r`n"), [Text.UTF8Encoding]::new($true))

$Archive = Join-Path $Sortie "$Nom.zip"
Remove-Item -LiteralPath $Archive -ErrorAction SilentlyContinue
Compress-Archive -Path (Join-Path $Sortie "$Nom/Sindry") -DestinationPath $Archive
$Empreinte = (Get-FileHash -LiteralPath $Archive -Algorithm SHA256).Hash
Write-Host "::notice::$Nom.zip : $Empreinte"
Add-Content -Path empreintes.txt -Value "$Nom.zip  SHA256  $Empreinte"

if ($ReleaseId) {
    $Uri = "https://uploads.github.com/repos/$env:GITHUB_REPOSITORY/releases/$ReleaseId/assets?name=$Nom.zip"
    $Entetes = @{ Authorization = "Bearer $env:GH_TOKEN"; Accept = 'application/vnd.github+json' }
    try {
        Invoke-RestMethod -Method Post -Uri $Uri -Headers $Entetes -ContentType 'application/zip' -InFile $Archive | Out-Null
    } catch {
        Write-Host "::error title=Version portable::l'archive n'a pas pu etre jointe a la release : $($_.Exception.Message)"
        exit 1
    }
    Write-Host "::notice::$Nom.zip jointe a la release $ReleaseId"
}

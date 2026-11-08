@echo off
setlocal DisableDelayedExpansion
set "LG_SETUP_FILE=%~f0"
set "LG_SETUP_SOURCE=%~1"
set "LG_SETUP_NONINTERACTIVE=%~2"
if not "%~3"=="" set "LG_SETUP_LANGUAGE=%~3"
if "%~1"=="" if not "%~2"=="--no-pause" if exist "%~dp0tools\windows_setup.ps1" (
    powershell.exe -NoLogo -NoProfile -STA -WindowStyle Hidden -ExecutionPolicy Bypass -File "%~dp0tools\windows_setup.ps1"
    exit /b
)
powershell.exe -NoLogo -NoProfile -Command "$text = [IO.File]::ReadAllText($env:LG_SETUP_FILE); & ([ScriptBlock]::Create(($text -split '# POWERSHELL_START\r?\n', 2)[1]))"
set "LG_SETUP_EXIT=%ERRORLEVEL%"
if not "%~2"=="--no-pause" pause
exit /b %LG_SETUP_EXIT%
# POWERSHELL_START
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetDirectoryName($env:LG_SETUP_FILE)
$private = Join-Path $root 'private'
$config = Join-Path $private 'data-path.txt'
$receipt = Join-Path $private 'setup.json'
$utf8 = New-Object System.Text.UTF8Encoding($false)
Add-Type -AssemblyName System.IO.Compression.FileSystem


# LANGUAGE_HELPERS_START
$requiredPacks = @('pak0.pk3', 'pak1_large.pk3', 'pak2.pk3', 'pak3.pk3', 'pak5_mod.pk3')
$languageNames = @('English', 'Deutsch', 'Francais', 'Espanol', 'Nederlands')
$languageCodes = @('en', 'de', 'fr', 'es', 'nl')
$languageFiles = @{ en = 'pak4_english.pk3'; de = 'pak4_german.pk3'; fr = 'pak4_french.pk3'; es = 'pak4_spanish.pk3'; nl = 'pak4_dutch.pk3' }
$languageBundleFiles = @{ de = 'base_g.pk3'; fr = 'base_f.pk3'; es = 'base_s.pk3'; nl = 'base_d.pk3' }
$languageWords = @{ en = 'step'; de = 'Schritt'; fr = 'pas'; es = 'paso'; nl = 'Terug' }

function Test-Language([string]$path, [string]$code) {
    if (-not $languageWords.ContainsKey($code)) { throw "Unknown language: $code" }
    $zip = [IO.Compression.ZipFile]::OpenRead($path)
    try {
        # Dutch contains menu translations only; the other packs contain dialog.
        $entryName = if ($code -eq 'nl') { 'ui/main.urc' } else { 'dialog/standard.tlk' }
        $entry = @($zip.Entries | Where-Object { $_.FullName.Replace('\', '/') -ieq $entryName }) | Select-Object -First 1
        if (-not $entry) { throw "$entryName is missing." }
        $reader = New-Object IO.StreamReader($entry.Open(), [Text.Encoding]::GetEncoding(1252), $false)
        try { $text = $reader.ReadToEnd() } finally { $reader.Dispose() }
        $word = [regex]::Escape($languageWords[$code])
        $pattern = if ($code -eq 'nl') {
            '(?im)^\s*title\s+"?' + $word + '"?\s*(?://[^\r\n]*)?$'
        } else {
            # Match a whole spoken word, excluding comments and quoted sound paths.
            '(?im)^(?![^\S\r\n]*(?://|"))[^\r\n]*\b' + $word + '\b'
        }
        if ($text -notmatch $pattern) {
            throw "Expected '$($languageWords[$code])' in $entryName."
        }
    } catch { throw "Language pack does not match $code`: $($_.Exception.Message)" }
    finally { $zip.Dispose() }
}

function Get-LanguagePaths([string]$base, [string]$code, [bool]$includeLoc = $false) {
    Join-Path $base $languageFiles[$code]
    if ($languageBundleFiles.ContainsKey($code)) {
        Join-Path $base $languageBundleFiles[$code]
        if ($includeLoc) { Join-Path $base ('loc\' + $languageBundleFiles[$code]) }
    }
}

function Get-ActiveLanguage([string]$base, [bool]$includeLoc = $false) {
    # Follow the same filename order as the engine for packs in the active base.
    $available = @()
    foreach ($code in $languageCodes) {
        foreach ($path in Get-LanguagePaths $base $code $includeLoc) {
            if (Test-Path -LiteralPath $path -PathType Leaf) {
                try {
                    Test-Language $path $code
                    $available += [PSCustomObject]@{ Code = $code; Path = $path; InBase = ([IO.Path]::GetDirectoryName($path) -ieq $base) }
                } catch { } # Invalid candidates do not make a folder complete.
            }
        }
    }
    $mounted = @($available | Where-Object { $_.InBase } | Sort-Object { [IO.Path]::GetFileName($_.Path).ToLowerInvariant() })
    # English supplies dialog only; it cannot override a translated menu.
    $translated = @($mounted | Where-Object { $_.Code -ne 'en' })
    if ($translated.Count) { return $translated[-1].Code }
    if ($mounted.Count) { return $mounted[-1].Code }
    if ($available.Count) { return $available[0].Code }
    return $null
}

function Test-DataFiles([string]$folder, [bool]$includeLoc = $false, [bool]$requireLanguage = $true) {
    foreach ($name in $requiredPacks) {
        if (-not (Test-Path -LiteralPath (Join-Path $folder $name) -PathType Leaf)) { return $false }
    }
    if ($requireLanguage) { return [bool](Get-ActiveLanguage $folder $includeLoc) }
    return $true
}

function Find-Base([string]$folder, [bool]$includeLoc = $true, [bool]$requireLanguage = $true) {
    if (-not $folder) { return $null }
    foreach ($suffix in @('', 'base', 'bin\base', 'Alice1\bin\base', 'alice_202106\Alice1\bin\base')) {
        $candidate = if ($suffix) { Join-Path $folder $suffix } else { $folder }
        if (Test-DataFiles $candidate $includeLoc $requireLanguage) { return [IO.Path]::GetFullPath($candidate) }
    }
    return $null
}
function Read-ConfiguredBase {
    $path = Join-Path $root 'private\data-path.txt'
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { return $null }
    $value = [IO.File]::ReadAllText($path).Trim()
    if (-not $value) { return $null }
    if (-not [IO.Path]::IsPathRooted($value)) { $value = Join-Path $root $value }
    return Find-Base $value $false
}

function Test-ManagedBase([string]$base) {
    if (-not $base) { return $false }
    $prefix = [IO.Path]::GetFullPath((Join-Path $root 'private\game-data')).TrimEnd('\') + '\'
    return [IO.Path]::GetFullPath($base).StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)
}

function Get-SavedLanguagePack([string]$code) {
    if (-not $languageFiles.ContainsKey($code)) { throw "Unknown language: $code" }
    $path = Join-Path $root ('private\game-data\languages\' + $languageFiles[$code])
    if (Test-Path -LiteralPath $path) {
        # An invalid saved variant is an error, never permission to replace it.
        Test-Language $path $code
        return $path
    }
    return $null
}

function Get-RequiredLanguages([string]$code) {
    if ($code -notin $languageCodes) { throw "Unknown language: $code" }
    $code
    if ($code -eq 'nl') { 'en' }
}

function Get-ReusableEnglish([string]$base, [string]$code) {
    if ($code -ne 'nl' -or -not $base) { return $null }
    $path = Join-Path $base $languageFiles['en']
    if (Test-Path -LiteralPath $path) {
        Test-Language $path 'en'
        return $path
    }
    return $null
}

function Get-LanguageAvailability([string]$code, [string]$base, [hashtable]$candidates, [bool]$download) {
    $missing = @()
    $allLocal = $true
    foreach ($required in Get-RequiredLanguages $code) {
        if ($required -eq 'en' -and (Get-ReusableEnglish $base $code)) { continue }
        if (Get-SavedLanguagePack $required) { continue }
        $allLocal = $false
        if (-not $download -and -not ($candidates -and $candidates.ContainsKey($required))) { $missing += $required }
    }
    return [PSCustomObject]@{ Available = $missing.Count -eq 0; AllLocal = $allLocal; Missing = $missing }
}

function Find-Extractor {
    $bundled = Join-Path $root 'setup\7zr.exe'
    if (Test-Path -LiteralPath $bundled -PathType Leaf) { return $bundled }
    foreach ($name in @('7z.exe', '7zz.exe')) {
        $command = Get-Command $name -CommandType Application -ErrorAction SilentlyContinue
        if ($command) { return $command.Source }
    }
    foreach ($parent in @($env:ProgramFiles, ${env:ProgramFiles(x86)})) {
        if ($parent) {
            $candidate = Join-Path $parent '7-Zip\7z.exe'
            if (Test-Path -LiteralPath $candidate -PathType Leaf) { return $candidate }
        }
    }
    throw 'To read .7z, install 7-Zip or extract the archive and choose its folder.'
}

function Get-ArchiveEntries([string]$source) {
    $extension = [IO.Path]::GetExtension($source).ToLowerInvariant()
    if ($extension -eq '.zip') {
        $zip = [IO.Compression.ZipFile]::OpenRead($source)
        try { $entries = @($zip.Entries | ForEach-Object { $_.FullName }) } finally { $zip.Dispose() }
    } elseif ($extension -eq '.7z') {
        $start = New-Object Diagnostics.ProcessStartInfo
        $start.FileName = Find-Extractor
        $start.Arguments = 'l -slt -ba -- "' + $source + '"'
        $start.UseShellExecute = $false; $start.CreateNoWindow = $true
        $start.RedirectStandardOutput = $true; $start.RedirectStandardError = $true
        $process = [Diagnostics.Process]::Start($start)
        try {
            $errors = $process.StandardError.ReadToEndAsync()
            $listing = $process.StandardOutput.ReadToEnd()
            $process.WaitForExit()
            if ($process.ExitCode -ne 0) { throw "Cannot list archive: $($errors.Result)" }
            $entries = @([regex]::Matches($listing, '(?m)^Path = (.+)\r?$') | ForEach-Object { $_.Groups[1].Value.TrimEnd([char]13) })
        } finally { $process.Dispose() }
    } else { throw 'Choose a ZIP or 7z archive.' }
    return $entries
}

function Get-SourceLanguages([string]$source) {
    $result = @{}
    if (-not $source) { return $result }
    $prefixes = @('', 'base/', 'loc/', 'bin/base/', 'Alice1/bin/base/', 'alice_202106/Alice1/bin/base/')
    if (Test-Path -LiteralPath $source -PathType Container) {
        foreach ($code in $languageCodes) {
            # Canonical names take priority across all supported folder structures.
            foreach ($name in @($languageFiles[$code], $languageBundleFiles[$code]) | Where-Object { $_ }) {
                foreach ($prefix in $prefixes) {
                    foreach ($suffix in @($prefix, ($prefix + 'loc/')) | Select-Object -Unique) {
                        $path = Join-Path $source ($suffix + $name)
                        if (Test-Path -LiteralPath $path -PathType Leaf) {
                            $result[$code] = [PSCustomObject]@{ Source = $source; Path = $path; Entry = $null }
                            break
                        }
                    }
                    if ($result.ContainsKey($code)) { break }
                }
                if ($result.ContainsKey($code)) { break }
            }
        }
        return $result
    }
    $entries = @(Get-ArchiveEntries $source)
    foreach ($code in $languageCodes) {
        foreach ($name in @($languageFiles[$code], $languageBundleFiles[$code]) | Where-Object { $_ }) {
            foreach ($prefix in $prefixes) {
                foreach ($suffix in @($prefix, ($prefix + 'loc/')) | Select-Object -Unique) {
                    $entry = $entries | Where-Object { $_.Replace('\', '/') -ieq ($suffix + $name) } | Select-Object -First 1
                    if ($entry) {
                        $result[$code] = [PSCustomObject]@{ Source = $source; Path = $null; Entry = $entry }
                        break
                    }
                }
                if ($result.ContainsKey($code)) { break }
            }
            if ($result.ContainsKey($code)) { break }
        }
    }
    return $result
}
# LANGUAGE_HELPERS_END

function Test-Cancelled {
    if ($env:LG_SETUP_CANCEL -and (Test-Path -LiteralPath $env:LG_SETUP_CANCEL)) {
        throw 'Setup cancelled. Your previous game-data setting and saves are unchanged.'
    }
}

function Set-Progress([string]$message, [int]$percent = -1) {
    Write-Host $message
    if ($env:LG_SETUP_STATUS) {
        Save-Text $env:LG_SETUP_STATUS ((@{ message = $message; percent = $percent } | ConvertTo-Json -Compress))
    }
    Test-Cancelled
}

function Test-Base([string]$folder, [bool]$requireLanguage = $true) {
    $maps = New-Object 'System.Collections.Generic.HashSet[string]' ([StringComparer]::OrdinalIgnoreCase)
    $foundVillage = $false
    foreach ($name in $requiredPacks) {
        Test-Cancelled
        $path = Join-Path $folder $name
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
            throw "Missing $name. Select the compatible 2011 base data or Alice1_2011_vanilla.7z."
        }
        $zip = [IO.Compression.ZipFile]::OpenRead($path)
        try {
            if ($zip.Entries.Count -eq 0) { throw "Empty game archive: $name" }
            foreach ($entry in $zip.Entries) {
                $entryName = $entry.FullName.Replace('\', '/').ToLowerInvariant()
                if ($entryName -match '^maps/[^/]+\.bsp$') { [void]$maps.Add($entryName) }
                if ($entryName -eq 'maps/gvillage.bsp') {
                    $stream = $entry.Open()
                    try {
                        $header = New-Object byte[] 8
                        $read = 0
                        while ($read -lt 8) {
                            $count = $stream.Read($header, $read, 8 - $read)
                            if ($count -eq 0) { throw 'Truncated village map.' }
                            $read += $count
                        }
                        if ([Text.Encoding]::ASCII.GetString($header, 0, 4) -ne 'FAKK' -or [BitConverter]::ToInt32($header, 4) -ne 42) {
                            throw 'The village map is not the supported FAKK version 42.'
                        }
                        $foundVillage = $true
                    } finally { $stream.Dispose() }
                }
            }
        } finally { $zip.Dispose() }
    }
    if (-not $foundVillage -or $maps.Count -ne 36) {
        throw "Expected the 36-map 2011 data set; found $($maps.Count) maps."
    }
    if ($requireLanguage -and -not (Get-ActiveLanguage $folder $true)) { throw 'Missing a supported language pack. Choose a folder or archive containing it.' }
    Write-Host 'Validated five base packs and 36 map entries.'
}

function Save-Text([string]$path, [string]$text) {
    $temporary = "$path.$([Guid]::NewGuid().ToString('N')).tmp"
    [IO.File]::WriteAllText($temporary, $text, $utf8)
    if (Test-Path -LiteralPath $path) {
        [IO.File]::Replace($temporary, $path, "$path.bak")
    } else {
        [IO.File]::Move($temporary, $path)
    }
}


function Save-LanguageCopy([string]$path, [string]$code) {
    $destination = Join-Path $languageCache $languageFiles[$code]
    if (Test-Path -LiteralPath $destination) { return }
    if ([IO.Path]::GetFullPath($path) -ieq [IO.Path]::GetFullPath($destination)) { return }
    [void][IO.Directory]::CreateDirectory($languageCache)
    $temporary = "$destination.$([Guid]::NewGuid().ToString('N')).tmp"
    [IO.File]::Copy($path, $temporary)
    try { [IO.File]::Move($temporary, $destination) }
    finally { if (Test-Path -LiteralPath $temporary) { [IO.File]::Delete($temporary) } }
}

function Copy-Language([string]$path, [string]$code) {
    Test-Language $path $code
    Save-LanguageCopy $path $code
}

function Import-Languages([string]$folder) {
    foreach ($code in $languageCodes) {
        if (Test-Path -LiteralPath (Join-Path $languageCache $languageFiles[$code])) { continue }
        foreach ($path in Get-LanguagePaths $folder $code $true) {
            if (Test-Path -LiteralPath $path -PathType Leaf) { Copy-Language $path $code; break }
        }
    }
}

function Import-SelectedLanguage([string]$source, [string]$code, [hashtable]$candidates = $null) {
    if (Get-SavedLanguagePack $code) { return }
    if ($null -eq $candidates) { $candidates = Get-SourceLanguages $source }
    $candidate = $candidates[$code]
    if (-not $candidate) { throw "The selected source does not contain the $code language pack." }
    if ($candidate.Path) { Copy-Language $candidate.Path $code; return }
    [void][IO.Directory]::CreateDirectory($languageCache)
    $temporary = Join-Path $languageCache ('.import-' + [Guid]::NewGuid().ToString('N'))
    [void][IO.Directory]::CreateDirectory($temporary)
    $target = Join-Path $temporary ([IO.Path]::GetFileName($candidate.Entry.Replace('/', '\')))
    try {
        Set-Progress "Extracting only the $code language pack..."
        if ([IO.Path]::GetExtension($source) -ieq '.zip') {
            $zip = [IO.Compression.ZipFile]::OpenRead($source)
            try {
                $entry = $zip.Entries | Where-Object { $_.FullName -ceq $candidate.Entry } | Select-Object -First 1
                [IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $target, $false)
            } finally { $zip.Dispose() }
        } else {
            $start = New-Object Diagnostics.ProcessStartInfo
            $start.FileName = Find-Extractor
            $start.UseShellExecute = $false; $start.CreateNoWindow = $true
            $start.Arguments = (@('e', '-y', '-bso0', '-bsp0', "-o$temporary", '--', $source, $candidate.Entry) | ForEach-Object { '"' + $_ + '"' }) -join ' '
            $process = [Diagnostics.Process]::Start($start)
            try {
                while (-not $process.WaitForExit(200)) { Test-Cancelled }
                if ($process.ExitCode -ne 0) { throw 'Language extraction failed.' }
            } finally {
                if (-not $process.HasExited) { $process.Kill(); $process.WaitForExit() }
                $process.Dispose()
            }
        }
        Test-Cancelled
        Copy-Language $target $code
    } finally {
        if (Test-Path -LiteralPath $target) { [IO.File]::Delete($target) }
        [IO.Directory]::Delete($temporary)
    }
}

function Prepare-Language([string]$source, [string]$base, [string]$code) {
    foreach ($required in Get-RequiredLanguages $code) {
        if ($required -eq 'en' -and (Get-ReusableEnglish $base $code)) { continue }
        Import-SelectedLanguage $source $required
    }
}

function Apply-Language([string]$base, [string]$code) {
    $prefix = [IO.Path]::GetFullPath((Join-Path $private 'game-data')).TrimEnd('\') + '\'
    if (-not ([IO.Path]::GetFullPath($base).StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase))) {
        throw 'Language destination is outside private/game-data.'
    }
    $script:languageTransaction = Join-Path $base ('.language-' + [Guid]::NewGuid().ToString('N'))
    [void][IO.Directory]::CreateDirectory($script:languageTransaction)
    $script:previousLanguageFiles = @()
    $script:installedLanguageFiles = @()
    $retainedEnglish = Get-ReusableEnglish $base $code
    $installCodes = @((Get-RequiredLanguages $code) | Where-Object { $_ -ne 'en' -or -not $retainedEnglish })
    # Prepare every target before moving any active pack.
    foreach ($required in $installCodes) {
        $selected = Get-SavedLanguagePack $required
        if (-not $selected) { throw "Missing $required language pack. Choose a source containing it." }
        $prepared = Join-Path $script:languageTransaction ("selected-$required.tmp")
        [IO.File]::Copy($selected, $prepared)
        [IO.File]::SetAttributes($prepared, [IO.FileAttributes]::Normal)
    }
    Test-Cancelled
    foreach ($other in $languageCodes) {
        if ($other -eq 'en' -and $retainedEnglish) { continue }
        foreach ($name in @($languageFiles[$other], $languageBundleFiles[$other]) | Where-Object { $_ }) {
            $path = Join-Path $base $name
            if (Test-Path -LiteralPath $path -PathType Leaf) {
                [IO.File]::Move($path, (Join-Path $script:languageTransaction $name))
                $script:previousLanguageFiles += $name
            }
        }
    }
    foreach ($required in $installCodes) {
        $destination = Join-Path $base $languageFiles[$required]
        [IO.File]::Move((Join-Path $script:languageTransaction ("selected-$required.tmp")), $destination)
        $script:installedLanguageFiles += $destination
    }
}

function Restore-Language {
    foreach ($path in $script:installedLanguageFiles) {
        if (Test-Path -LiteralPath $path) { [IO.File]::Delete($path) }
    }
    foreach ($name in $script:previousLanguageFiles) {
        [IO.File]::Move((Join-Path $script:languageTransaction $name), (Join-Path $base $name))
    }
}

function Complete-LanguageTransaction {
    # Only remove files belonging to this checked, managed transaction.
    foreach ($item in Get-ChildItem -LiteralPath $script:languageTransaction -File) {
        [IO.File]::SetAttributes($item.FullName, [IO.FileAttributes]::Normal)
        [IO.File]::Delete($item.FullName)
    }
    [IO.Directory]::Delete($script:languageTransaction)
}

function Switch-InstalledLanguage {
    $base = Read-ConfiguredBase
    if (-not $base) { throw 'No complete installation is configured. Use Set up first.' }
    if (-not (Test-ManagedBase $base)) { throw 'This installation uses an external folder. Use Set up to copy it into private/game-data first.' }
    $code = $env:LG_SETUP_LANGUAGE
    if ($code -notin $languageCodes) { throw "Unknown language: $code" }
    if ((Get-ActiveLanguage $base) -eq $code) { throw 'The selected language is already active.' }
    Set-Progress 'Preparing language change for the existing installation...'
    # Resolve before backing up displaced packs; a saved variant always wins.
    Prepare-Language $env:LG_SETUP_SOURCE $base $code
    foreach ($other in $languageCodes) {
        if (Test-Path -LiteralPath (Join-Path $languageCache $languageFiles[$other])) { continue }
        foreach ($path in Get-LanguagePaths $base $other) {
            if (Test-Path -LiteralPath $path -PathType Leaf) { Copy-Language $path $other; break }
        }
    }
    if ((Get-ActiveLanguage $base) -eq $code) { throw 'The selected language is already active.' }
    Test-Cancelled
    try {
        Apply-Language $base $code
    } catch {
        Restore-Language
        throw
    }
    # The change is committed. A cleanup failure must not roll back partially
    # deleted backups; leftover files are outside the mounted base file list.
    try { Complete-LanguageTransaction } catch { Write-Host 'Language changed; a temporary backup folder could not be removed.' }
    Write-Host "Language changed to $code in $base. No base-data import was performed."
}

try {
    $languageCache = Join-Path $private 'game-data\languages'
    $action = if ($env:LG_SETUP_ACTION) { $env:LG_SETUP_ACTION } else { 'setup' }
    if ($action -eq 'language') { Switch-InstalledLanguage; exit 0 }
    if ($action -ne 'setup') { throw "Unknown setup action: $action" }
    Write-Host 'Looking Glass - local game-data setup'
    Write-Host 'Use an existing Alice1 folder or a compatible .7z/.zip archive.'
    Write-Host 'Download page: https://archive.org/details/alice_202106'
    Write-Host 'File: Alice1_2011_vanilla.7z (use game data you are entitled to use).'
    Write-Host ''
    $source = $env:LG_SETUP_SOURCE
    if ([string]::IsNullOrWhiteSpace($source)) {
        $source = Find-Base $root
        if (-not $source -and (Test-Path -LiteralPath $config)) {
            $saved = [IO.File]::ReadAllText($config).Trim()
            if ($saved) {
                $candidate = if ([IO.Path]::IsPathRooted($saved)) { $saved } else { Join-Path $root $saved }
                $source = Find-Base $candidate $true $false
            }
        }
        if (-not $source) {
            foreach ($candidate in @((Join-Path $root 'Alice1_2011_vanilla.7z'), (Join-Path $root 'alice_202106\Alice1_2011_vanilla.7z'))) {
                if (Test-Path -LiteralPath $candidate -PathType Leaf) { $source = $candidate; break }
            }
        }
        if (-not $source) {
            if ($env:LG_SETUP_NONINTERACTIVE -eq '--no-pause') { throw 'No game data found. Pass an archive or folder as the first argument.' }
            $source = Read-Host 'Paste the downloaded .7z path or extracted Alice1 folder (blank cancels)'
        }
    }
    $source = ([string]$source).Trim().Trim('"')
    if (-not $source) { throw 'Setup cancelled; existing configuration was kept.' }
    $source = (Get-Item -LiteralPath $source).FullName
    $archiveHash = $null
    $languageCache = Join-Path $private 'game-data\languages'
    $language = $env:LG_SETUP_LANGUAGE
    $base = $null
    if (Test-Path -LiteralPath $source -PathType Container) {
        $base = Find-Base $source $true $false
        if (-not $base) { throw 'No complete base data found. Provide the five base packs plus a supported language pack, and select a source containing the chosen language, or use a saved language pack.' }
    } else {
        $extension = [IO.Path]::GetExtension($source).ToLowerInvariant()
        if ($extension -notin @('.7z', '.zip')) { throw 'Select a .7z or .zip archive, or an extracted game folder.' }
        if ($extension -eq '.7z') { $extractor = Find-Extractor }
        Set-Progress 'Checking the selected archive...'
        $hasher = [Security.Cryptography.SHA256]::Create()
        $archiveStream = [IO.File]::OpenRead($source)
        try {
            $buffer = New-Object byte[] (1024 * 1024)
            while (($count = $archiveStream.Read($buffer, 0, $buffer.Length)) -gt 0) {
                Test-Cancelled
                [void]$hasher.TransformBlock($buffer, 0, $count, $null, 0)
            }
            [void]$hasher.TransformFinalBlock($buffer, 0, 0)
            $archiveHash = ([BitConverter]::ToString($hasher.Hash)).Replace('-', '').ToLowerInvariant()
        }
        finally { $archiveStream.Dispose(); $hasher.Dispose() }
        if (Test-Path -LiteralPath $receipt) {
            try {
                $previous = [IO.File]::ReadAllText($receipt) | ConvertFrom-Json
                if ($previous.archiveSha256 -eq $archiveHash) {
                    $candidate = Join-Path $root $previous.dataPath
                    if (Test-Path -LiteralPath $candidate -PathType Container) {
                        Test-Base $candidate
                        $base = $candidate
                        Write-Host 'Reusing the already imported copy of this download.'
                    }
                }
            } catch { Write-Host 'The previous import cannot be reused; making a fresh import.' }
        }
        if (-not $base) {
            # Exact entry names plus flat extraction exclude all original executables,
            # scripts, traversal paths and language subfolders. Never overwrite input.
            $relative = 'private\game-data\import-' + [Guid]::NewGuid().ToString('N') + '\base'
            $base = Join-Path $root $relative
            [void][IO.Directory]::CreateDirectory($base)
            Set-Progress 'Unpacking game files. This may take a few minutes...'
            $archiveEntries = @(Get-ArchiveEntries $source)
            $prefix = $null
            foreach ($candidate in @('', 'base/', 'bin/base/', 'Alice1/bin/base/', 'alice_202106/Alice1/bin/base/')) {
                if (@($archiveEntries | Where-Object { $_.Replace('\', '/') -ieq ($candidate + 'pak0.pk3') }).Count) { $prefix = $candidate; break }
            }
            if ($null -eq $prefix) { throw 'No supported base folder in archive.' }
            $wanted = @($requiredPacks) + @($languageFiles.Values) + @($languageBundleFiles.Values)
            $entries = @()
            foreach ($name in $wanted) {
                $entry = $archiveEntries | Where-Object { $_.Replace('\', '/') -ieq ($prefix + $name) } | Select-Object -First 1
                if (-not $entry -and $name -in $languageBundleFiles.Values) {
                    $entry = $archiveEntries | Where-Object { $_.Replace('\', '/') -ieq ($prefix + 'loc/' + $name) } | Select-Object -First 1
                }
                if ($entry) { $entries += $entry }
            }
            if ($extension -eq '.zip') {
                $zip = [IO.Compression.ZipFile]::OpenRead($source)
                try {
                    foreach ($name in $entries) {
                        Test-Cancelled
                        $entry = $zip.Entries | Where-Object { $_.FullName -ceq $name } | Select-Object -First 1
                        [IO.Compression.ZipFileExtensions]::ExtractToFile($entry, (Join-Path $base ([IO.Path]::GetFileName($name.Replace('/', '\')))), $false)
                    }
                } finally { $zip.Dispose() }
            } else {
            $start = New-Object Diagnostics.ProcessStartInfo
            $start.FileName = $extractor
            $start.UseShellExecute = $false
            $start.CreateNoWindow = $true
            # Paths are literal Windows paths, which cannot contain double quotes.
            $start.Arguments = (@('e', '-y', '-bso0', '-bsp0', "-o$base", '--', $source) + $entries | ForEach-Object { '"' + $_ + '"' }) -join ' '
            $extract = [Diagnostics.Process]::Start($start)
            try {
                while (-not $extract.WaitForExit(200)) { Test-Cancelled }
                if ($extract.ExitCode -ne 0) { throw 'Unpacking failed. Check the archive and free disk space, then try again.' }
            } finally {
                if (-not $extract.HasExited) { $extract.Kill(); $extract.WaitForExit() }
                $extract.Dispose()
            }
            }
        }
    }
    Set-Progress 'Validating game files...'
    Test-Base $base $false
    if (-not $language) { $language = Get-ActiveLanguage $base $true }
    if ($language -notin $languageCodes) { throw "Unknown language: $language" }
    # Resolve the selected source before importing staged/previous base packs.
    # This preserves canonical-name priority even when reusing an old import.
    Prepare-Language $source $base $language
    $sourceLanguages = Get-SourceLanguages $source
    foreach ($code in $languageCodes) {
        if (-not (Test-Path -LiteralPath (Join-Path $languageCache $languageFiles[$code])) -and $sourceLanguages.ContainsKey($code)) {
            Import-SelectedLanguage $source $code $sourceLanguages
        }
    }
    Import-Languages $base
    Test-Cancelled
    $managedPrefix = [IO.Path]::GetFullPath((Join-Path $private 'game-data')).TrimEnd('\') + '\'
    if (-not $base.StartsWith($managedPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        $managed = Join-Path $private ('game-data\import-' + [Guid]::NewGuid().ToString('N') + '\base')
        [void][IO.Directory]::CreateDirectory($managed)
        foreach ($pack in Get-ChildItem -LiteralPath $base -File -Filter '*.pk3') {
            Test-Cancelled
            [IO.File]::Copy($pack.FullName, (Join-Path $managed $pack.Name))
        }
        $base = $managed
    }
    [void][IO.Directory]::CreateDirectory($private)
    $oldConfig = if (Test-Path -LiteralPath $config) { [IO.File]::ReadAllText($config) } else { $null }
    $oldReceipt = if (Test-Path -LiteralPath $receipt) { [IO.File]::ReadAllText($receipt) } else { $null }
    try {
        Apply-Language $base $language
        $rootPrefix = [IO.Path]::GetFullPath($root).TrimEnd('\') + '\'
        $savedPath = $base.Substring($rootPrefix.Length)
        Save-Text $receipt (([ordered]@{ schema = 1; source = $source; archiveSha256 = $archiveHash; dataPath = $savedPath } | ConvertTo-Json) + [Environment]::NewLine)
        Save-Text $config ($savedPath + [Environment]::NewLine)
    } catch {
        Restore-Language
        if ($null -ne $oldReceipt) { Save-Text $receipt $oldReceipt } elseif (Test-Path -LiteralPath $receipt) { [IO.File]::Delete($receipt) }
        if ($null -ne $oldConfig) { Save-Text $config $oldConfig } elseif (Test-Path -LiteralPath $config) { [IO.File]::Delete($config) }
        throw
    }
    Complete-LanguageTransaction
    Write-Host ''
    Write-Host "Game data configured: $base"
    Write-Host 'Ready. Double-click Launch.cmd to play.'
    exit 0
} catch {
    Write-Host ''
    Write-Host ("Setup could not finish: " + $_.Exception.Message) -ForegroundColor Red
    exit 1
}

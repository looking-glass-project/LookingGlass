param(
    [ValidateSet('Setup', 'Play', 'Worker')][string]$Mode = 'Setup',
    [string]$JobFile
)
# Windows PowerShell 5.1; no installed modules, administrator rights or Python.
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$utf8 = New-Object Text.UTF8Encoding($false)
$archiveName = 'Alice1_2011_vanilla.7z'
$downloadUrl = 'https://archive.org/download/alice_202106/Alice1_2011_vanilla.7z'
$downloadBytes = 977811158L
$downloadHash = 'b253bb2c9c875f1838a711d0fe4f12f4bc2c6207db3baaE6431af962eca8705d'.ToLowerInvariant()
Add-Type -AssemblyName System.IO.Compression.FileSystem
$setupText = [IO.File]::ReadAllText((Join-Path $root 'Setup.cmd'))
$languageHelpers = ($setupText -split '# LANGUAGE_HELPERS_START\r?\n', 2)[1] -split '# LANGUAGE_HELPERS_END', 2
. ([ScriptBlock]::Create($languageHelpers[0]))


function Save-State([string]$path, [object]$value) {
    $temp = $path + '.' + [Guid]::NewGuid().ToString('N') + '.tmp'
    [IO.File]::WriteAllText($temp, ($value | ConvertTo-Json -Compress), $utf8)
    if (Test-Path -LiteralPath $path) { [IO.File]::Replace($temp, $path, ($path + '.bak')) }
    else { [IO.File]::Move($temp, $path) }
}

function Find-Data([string]$folder, [bool]$includeLoc = $true) {
    return Find-Base $folder $includeLoc
}

function Read-ConfiguredData {
    return Read-ConfiguredBase
}

function Start-Game {
    $exe = Join-Path $root 'looking-glass.exe'
    if (-not (Test-Path -LiteralPath $exe -PathType Leaf)) {
        throw 'Game files are ready. Close setup and use Launch.cmd to start your development build.'
    }
    $start = New-Object Diagnostics.ProcessStartInfo
    $start.FileName = $exe
    $start.WorkingDirectory = $root
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    [void][Diagnostics.Process]::Start($start)
}

if ($Mode -eq 'Worker') {
    $job = [IO.File]::ReadAllText($JobFile) | ConvertFrom-Json
    $statusPath = Join-Path $job.directory 'status.json'
    $cancelPath = Join-Path $job.directory 'cancel'
    function Check-Cancel {
        if (Test-Path -LiteralPath $cancelPath) { throw 'Setup cancelled. Previous settings and saves were kept.' }
    }
    function Report([string]$message, [int]$percent = -1) {
        Save-State $statusPath @{ message = $message; percent = $percent }
        Check-Cancel
    }
    function Download-Hash([string]$path) {
        $stream = [IO.File]::OpenRead($path)
        $hash = [Security.Cryptography.SHA256]::Create()
        try {
            $buffer = New-Object byte[] (1024 * 1024)
            while (($count = $stream.Read($buffer, 0, $buffer.Length)) -gt 0) {
                Check-Cancel
                [void]$hash.TransformBlock($buffer, 0, $count, $null, 0)
            }
            [void]$hash.TransformFinalBlock($buffer, 0, 0)
            return [BitConverter]::ToString($hash.Hash).Replace('-', '').ToLowerInvariant()
        } finally { $stream.Dispose(); $hash.Dispose() }
    }
    try {
        Check-Cancel
        $action = if ($job.action) { [string]$job.action } else { 'setup' }
        if ($action -notin @('setup', 'language')) { throw "Unknown setup action: $action" }
        $source = $job.source
        if ($job.download -isnot [bool]) { throw 'Choose local files or explicitly opt in to downloading.' }
        if ($job.download -and ($job.downloadConfirmed -isnot [bool] -or -not $job.downloadConfirmed)) {
            # A selected radio button (or a stale job from an older setup) is not consent.
            throw 'Download not confirmed. Only download game files you are entitled to use; confirm in Setup first.'
        }
        $languageAvailability = $null
        if ($action -eq 'language') {
            $installed = Read-ConfiguredBase
            if (-not (Test-ManagedBase $installed)) { throw 'A managed installation is required. Use Set up first.' }
            if ((Get-ActiveLanguage $installed) -eq $job.language) { throw 'The selected language is already active.' }
            $languageAvailability = Get-LanguageAvailability $job.language $installed @{} $true
        }
        if ($job.download -and ($action -eq 'setup' -or -not $languageAvailability.AllLocal)) {
            $downloads = Join-Path $root 'private\downloads'
            [void][IO.Directory]::CreateDirectory($downloads)
            $source = Join-Path $downloads $archiveName
            if (-not (Test-Path -LiteralPath $source)) {
                $partial = Join-Path $downloads ([Guid]::NewGuid().ToString('N') + '.partial')
                Report 'Connecting to Internet Archive...'
                [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
                $request = [Net.HttpWebRequest]::Create($downloadUrl)
                $request.UserAgent = 'LookingGlass-Setup/1.0'
                $request.Timeout = 20000
                $request.ReadWriteTimeout = 15000
                $response = $request.GetResponse()
                try {
                    if ($response.ResponseUri.Scheme -ne 'https') { throw 'Download was redirected away from HTTPS.' }
                    $input = $response.GetResponseStream()
                    $output = [IO.File]::Create($partial)
                    try {
                        $buffer = New-Object byte[] (256 * 1024)
                        $received = 0L
                        $clock = [Diagnostics.Stopwatch]::StartNew()
                        while (($count = $input.Read($buffer, 0, $buffer.Length)) -gt 0) {
                            Check-Cancel
                            $output.Write($buffer, 0, $count)
                            $received += $count
                            if ($received -gt $downloadBytes) { throw 'The download has changed. Use Choose archive or Choose folder instead.' }
                            if ($clock.ElapsedMilliseconds -gt 300) {
                                Report ('Downloading game files: {0:N0} of 933 MB' -f ($received / 1MB)) ([Math]::Min(100, [int](100 * $received / $downloadBytes)))
                                $clock.Restart()
                            }
                        }
                    } finally { $output.Dispose(); $input.Dispose() }
                } finally { $response.Close() }
                Check-Cancel
                if ((Get-Item -LiteralPath $partial).Length -ne $downloadBytes) { throw 'The download is incomplete. Please try again, or choose a local archive.' }
                Report 'Checking the downloaded file...'
                if ((Download-Hash $partial) -ne $downloadHash) { throw 'The download did not pass its integrity check. It has not been installed.' }
                Check-Cancel
                [IO.File]::Move($partial, $source)
            } else {
                Report 'Checking the previously downloaded file...'
                if ((Download-Hash $source) -ne $downloadHash) { throw 'The saved download did not pass its integrity check. Choose a different archive or folder.' }
            }
        }
        Check-Cancel
        $env:LG_SETUP_FILE = Join-Path $root 'Setup.cmd'
        $env:LG_SETUP_ACTION = $action
        $env:LG_SETUP_SOURCE = $source
        $env:LG_SETUP_LANGUAGE = $job.language
        $env:LG_SETUP_NONINTERACTIVE = '--no-pause'
        $env:LG_SETUP_STATUS = $statusPath
        $env:LG_SETUP_CANCEL = $cancelPath
        $backend = ([IO.File]::ReadAllText($env:LG_SETUP_FILE) -split '# POWERSHELL_START\r?\n', 2)[1]
        & ([ScriptBlock]::Create($backend)) *>&1 | Out-File -LiteralPath (Join-Path $job.directory 'setup.log') -Encoding utf8
        exit $LASTEXITCODE
    } catch {
        Save-State $statusPath @{ message = $_.Exception.Message; percent = 0; failed = $true }
        exit 1
    } finally {
        # Only the partial file created by this worker is removed; never the user's archive.
        if ($partial -and (Test-Path -LiteralPath $partial)) { [IO.File]::Delete($partial) }
    }
}

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
[Windows.Forms.Application]::EnableVisualStyles()

# Serialize setup per installation, including a shortcut opened twice.
$rootHash = [Security.Cryptography.SHA256]::Create()
try { $key = [BitConverter]::ToString($rootHash.ComputeHash([Text.Encoding]::UTF8.GetBytes($root.ToLowerInvariant()))).Replace('-', '') }
finally { $rootHash.Dispose() }
$mutex = New-Object Threading.Mutex($false, ('Local\LookingGlassSetup-' + $key))
try { $owned = $mutex.WaitOne(0) } catch [Threading.AbandonedMutexException] { $owned = $true }
if (-not $owned) {
    [void][Windows.Forms.MessageBox]::Show('Setup is already open for this game folder.', 'Looking Glass')
    exit 0
}

try {
    if ($Mode -eq 'Play' -and (Read-ConfiguredData)) { Start-Game; exit 0 }
    $form = New-Object Windows.Forms.Form
    $form.Text = 'Looking Glass - Setup'
    $form.ClientSize = New-Object Drawing.Size(660, 760)
    $form.StartPosition = 'CenterScreen'
    $form.FormBorderStyle = 'FixedDialog'
    $form.MaximizeBox = $false
    $form.Font = New-Object Drawing.Font('Segoe UI', 10)
    $form.AutoScaleMode = 'Dpi'

    function Label([string]$text, [int]$x, [int]$y, [int]$w, [int]$h) {
        $c = New-Object Windows.Forms.Label
        $c.Text = $text; $c.SetBounds($x, $y, $w, $h); $form.Controls.Add($c); return $c
    }
    function Button([string]$text, [int]$x, [int]$y, [int]$w) {
        $c = New-Object Windows.Forms.Button
        $c.Text = $text; $c.SetBounds($x, $y, $w, 38); $form.Controls.Add($c); return $c
    }
    $heading = Label 'Welcome to Looking Glass' 24 20 610 40
    $heading.Font = New-Object Drawing.Font('Segoe UI', 20, [Drawing.FontStyle]::Bold)
    $intro = Label 'Set up game files or change the language. Open Launch to play.' 26 70 610 45
    $local = New-Object Windows.Forms.RadioButton
    $local.Text = 'Use game files already on this computer'; $local.SetBounds(26, 120, 605, 28)
    $form.Controls.Add($local)
    $choices = New-Object Windows.Forms.ComboBox
    $choices.DropDownStyle = 'DropDownList'; $choices.SetBounds(44, 155, 570, 30)
    $choices.AccessibleName = 'Game file location'
    $choices.DropDownWidth = 700; $form.Controls.Add($choices)
    $archiveButton = Button 'Choose archive...' 44 193 170
    $folderButton = Button 'Choose folder...' 224 193 170
    $download = New-Object Windows.Forms.RadioButton
    $download.Text = 'Download from Internet Archive (optional, 933 MB)'; $download.SetBounds(26, 244, 605, 28)
    $form.Controls.Add($download)
    $sourceNote = Label 'Only if you are entitled to download and use these files. Confirmation is required. Allow 3 GB of free space.' 44 276 570 46

    $languageLabel = Label 'Language' 26 330 120 25
    $languages = New-Object Windows.Forms.ComboBox
    $languages.DropDownStyle = 'DropDownList'; $languages.SetBounds(150, 328, 250, 30)
    $languages.AccessibleName = 'Game language'
    [void]$languages.Items.AddRange([object[]]$languageNames)
    $form.Controls.Add($languages)
    $languageHint = Label 'Languages come from the selected source and the saved language packs. Saved packs take priority.' 26 365 605 45
    $installationLabel = Label '' 26 410 605 54
    $languageStatus = Label '' 26 470 605 60
    $script:configured = Read-ConfiguredData
    $activeCode = if ($script:configured) { Get-ActiveLanguage $script:configured } else { 'en' }
    $languages.SelectedIndex = [Array]::IndexOf($languageCodes, $activeCode)

    $desktop = New-Object Windows.Forms.CheckBox
    $desktop.Text = 'Create a desktop shortcut'; $desktop.SetBounds(26, 535, 290, 28)
    $form.Controls.Add($desktop)
    $playAfter = New-Object Windows.Forms.CheckBox
    $playAfter.Text = 'Play when setup finishes'; $playAfter.SetBounds(330, 535, 290, 28)
    $playAfter.Checked = Test-Path -LiteralPath (Join-Path $root 'looking-glass.exe')
    $form.Controls.Add($playAfter)
    $progress = New-Object Windows.Forms.ProgressBar
    $progress.AccessibleName = 'Setup progress'
    $progress.SetBounds(26, 572, 605, 20); $form.Controls.Add($progress)
    $statusLabel = Label 'Choose an option above, then select Set up.' 26 602 605 45
    $operationNote = Label '' 26 655 605 40
    $go = Button 'Set up' 216 704 140
    $applyLanguage = Button 'Apply language' 366 704 160
    $cancel = Button 'Close' 536 704 95
    $form.AcceptButton = $go
    $form.CancelButton = $cancel

    # Keep actions visible on short desktops; only the setup details scroll.
    $form.SuspendLayout()
    $footer = New-Object Windows.Forms.Panel
    $footer.Dock = 'Bottom'; $footer.Height = 64
    $content = New-Object Windows.Forms.Panel
    $content.Dock = 'Fill'; $content.AutoScroll = $true
    $content.AutoScrollMinSize = New-Object Drawing.Size(0, 696)
    foreach ($control in @($form.Controls)) {
        if (@($go, $applyLanguage, $cancel) -contains $control) {
            $control.Top = 14
            $footer.Controls.Add($control)
        } else {
            $content.Controls.Add($control)
        }
    }
    $form.Controls.Add($content)
    $form.Controls.Add($footer)
    $workArea = [Windows.Forms.Screen]::FromControl($form).WorkingArea
    $form.Height = [Math]::Min($form.Height, $workArea.Height - 16)
    $form.ResumeLayout($true)

    $timer = New-Object Windows.Forms.Timer
    $timer.Interval = 250
    $script:worker = $null
    $script:closeWhenDone = $false
    $script:ready = $false
    $script:currentJob = $null
    $script:currentAction = 'setup'

    function Add-Choice([string]$path) {
        if ($path -and (Test-Path -LiteralPath $path)) {
            $index = $choices.Items.IndexOf($path)
            if ($index -lt 0) { $index = $choices.Items.Add($path) }
            $choices.SelectedIndex = $index
        }
    }
    foreach ($folder in @($root, (Join-Path $env:USERPROFILE 'Downloads'))) {
        Add-Choice (Find-Data $folder)
        Add-Choice (Join-Path $folder $archiveName)
    }
    Add-Choice (Join-Path $root ('alice_202106\' + $archiveName))
    Add-Choice (Join-Path $root ('private\downloads\' + $archiveName))
    Add-Choice (Read-ConfiguredData)
    # Missing local data must never select or initiate a third-party download.
    $local.Checked = $true
    $download.Checked = $false
    if ($choices.Items.Count -gt 0) { $statusLabel.Text = 'Found game files on this computer. Select Set up to connect them.' }
    else { $statusLabel.Text = 'Choose an archive or folder containing game files you are entitled to use.' }

    function Refresh-Inputs {
        $busy = $null -ne $script:worker
        $local.Enabled = -not $busy; $download.Enabled = -not $busy
        $choices.Enabled = -not $busy -and $local.Checked
        $archiveButton.Enabled = -not $busy; $folderButton.Enabled = -not $busy
        $languages.Enabled = -not $busy
        $applyLanguage.Enabled = -not $busy -and $script:canApply
        $go.Enabled = -not $busy -and ($script:ready -or $download.Checked -or ($local.Checked -and $choices.SelectedIndex -ge 0))
        $desktop.Enabled = -not $busy; $playAfter.Enabled = -not $busy
    }
    $script:sourceLanguages = @{}
    $script:sourceError = $null
    $script:sourceScanning = $false
    $script:sourceScan = $null
    $script:sourceScanHandle = $null
    $script:sourceKey = $null
    $script:canApply = $false
    $sourceTimer = New-Object Windows.Forms.Timer
    $sourceTimer.Interval = 200

    function Refresh-Language {
        $code = $languageCodes[$languages.SelectedIndex]
        $source = [string]$choices.SelectedItem
        $managed = Test-ManagedBase $script:configured
        $active = if ($script:configured) { Get-ActiveLanguage $script:configured } else { $null }
        $availability = $null
        $packError = $null
        $incompleteDutch = $false
        try {
            $availability = Get-LanguageAvailability $code $script:configured $script:sourceLanguages $download.Checked
            if ($active -eq 'nl') { $incompleteDutch = -not (Test-Path -LiteralPath (Join-Path $script:configured $languageFiles['en']) -PathType Leaf) }
        } catch { $packError = $_.Exception.Message }
        $script:canApply = $managed -and $active -ne $code -and $availability -and $availability.Available -and -not $packError
        if ($script:configured) {
            $activeName = $languageNames[[Array]::IndexOf($languageCodes, $active)]
            $installationLabel.Text = "Installed language: $activeName. Language changes apply to:`r`n$script:configured"
        } else {
            $installationLabel.Text = 'No complete installation is configured. Use Set up to install game files first.'
        }
        if ($packError) {
            $languageStatus.Text = "Required pack cannot be used: $packError No source pack will silently replace a saved variant."
        } elseif ($script:configured -and -not $managed) {
            $languageStatus.Text = 'This installation uses an external folder. Use Set up to copy it into private/game-data before changing its language.'
        } elseif ($active -eq $code) {
            $languageStatus.Text = if ($incompleteDutch) {
                'Nederlands is active but pak4_english.pk3 is missing. Switch to another language, then back to Nederlands to complete the pair.'
            } else { 'This language is already active. Select another language to apply a change.' }
        } elseif ($availability.AllLocal) {
            $languageStatus.Text = 'All required packs are saved or reusable in base. Apply language needs no download or extraction.'
        } elseif ($download.Checked) {
            $languageStatus.Text = 'Available from download. Apply language imports only the required language packs; Nederlands needs Dutch and English.'
        } elseif ($availability.Available) {
            $languageStatus.Text = 'Available from saved packs and the selected source. Nederlands needs Dutch and English. Only required language packs are imported.'
        } elseif ($script:sourceScanning) {
            $languageStatus.Text = 'Checking the selected source for language packs...'
        } elseif ($script:sourceError) {
            $languageStatus.Text = "Cannot read the selected source: $script:sourceError"
        } else {
            $missingNames = @($availability.Missing | ForEach-Object { $languageFiles[$_] }) -join ', '
            $languageStatus.Text = "Required pack unavailable: $missingNames. Choose a folder, archive or download containing it."
        }
        $go.Text = if ($script:ready) { 'Play now' } else { 'Set up' }
        $operationNote.Text = if ($script:ready) { 'Play now: start the configured game.' } elseif ($download.Checked) {
            'Set up: download compatible game files (933 MB), or reuse the saved download, then import them.'
        } elseif ($source -and [IO.Path]::GetExtension($source) -in @('.7z', '.zip')) {
            'Set up: import the selected local archive and configure its game files. No download.'
        } elseif ($source -and $managed -and [IO.Path]::GetFullPath($source) -ieq [IO.Path]::GetFullPath($script:configured)) {
            'Set up: validate and configure the selected installed data, including the selected language. No download.'
        } else {
            'Set up: import the selected local folder into private/game-data and configure its game files. No download.'
        }
        Refresh-Inputs
    }
    function Start-SourceScan {
        $source = if ($local.Checked) { [string]$choices.SelectedItem } else { $null }
        $key = if ($download.Checked) { 'download' } else { $source }
        if ($key -eq $script:sourceKey) { return }
        $script:sourceKey = $key
        $script:sourceLanguages = @{}
        $script:sourceError = $null
        if ($script:sourceScan) {
            $script:sourceScan.Stop()
            $script:sourceScan.Dispose()
            $script:sourceScan = $null
        }
        $script:sourceScanning = [bool]$source
        if (-not $source) { return }
        # A background runspace reads folder/ZIP/7z indexes; it never imports files.
        $script:sourceScan = [PowerShell]::Create()
        [void]$script:sourceScan.AddScript({
            param($helpers, $installationRoot, $selectedSource)
            $ErrorActionPreference = 'Stop'
            $root = $installationRoot
            Add-Type -AssemblyName System.IO.Compression.FileSystem
            . ([ScriptBlock]::Create($helpers))
            Get-SourceLanguages $selectedSource
        }).AddArgument($languageHelpers[0]).AddArgument($root).AddArgument($source)
        $script:sourceScanHandle = $script:sourceScan.BeginInvoke()
        $sourceTimer.Start()
    }
    $sourceTimer.Add_Tick({
        if (-not $script:sourceScan) { $sourceTimer.Stop(); return }
        if ($script:sourceScanHandle.IsCompleted) {
            try {
                $results = $script:sourceScan.EndInvoke($script:sourceScanHandle)
                if ($script:sourceScan.HadErrors) { throw $script:sourceScan.Streams.Error[0].ToString() }
                if ($results.Count) { $script:sourceLanguages = $results[0] }
            } catch { $script:sourceError = $_.Exception.Message }
            finally {
                $script:sourceScan.Dispose(); $script:sourceScan = $null
                $script:sourceScanning = $false; $sourceTimer.Stop()
            }
            Refresh-Language
        }
    })
    function Setup-SelectionChanged {
        $script:ready = $false
        Start-SourceScan
        Refresh-Language
    }
    $languages.Add_SelectedIndexChanged({ Setup-SelectionChanged })
    $local.Add_CheckedChanged({ Setup-SelectionChanged })
    $download.Add_CheckedChanged({ Setup-SelectionChanged })
    $choices.Add_SelectedIndexChanged({ Setup-SelectionChanged })
    $archiveButton.Add_Click({
        $dialog = New-Object Windows.Forms.OpenFileDialog
        $dialog.Title = 'Choose a game or language archive'
        $dialog.Filter = 'Game archive (*.7z;*.zip)|*.7z;*.zip'
        if ($dialog.ShowDialog($form) -eq 'OK') { Add-Choice $dialog.FileName; $local.Checked = $true; Setup-SelectionChanged }
        $dialog.Dispose()
    })
    $folderButton.Add_Click({
        $dialog = New-Object Windows.Forms.FolderBrowserDialog
        $dialog.Description = 'Choose Alice1, bin\base, or a folder containing language packs.'
        $dialog.ShowNewFolderButton = $false
        if ($dialog.ShowDialog($form) -eq 'OK') { Add-Choice $dialog.SelectedPath; $local.Checked = $true; Setup-SelectionChanged }
        $dialog.Dispose()
    })
    function Start-SetupWorker([string]$action) {
        if ($script:worker) { return }
        if ($action -eq 'setup' -and $local.Checked -and $choices.SelectedIndex -lt 0) { throw 'Choose an archive or folder first.' }
        $downloadRequested = $download.Checked
        if ($action -eq 'language') {
            $script:configured = Read-ConfiguredData
            Refresh-Language
            if (-not $script:canApply) { throw 'Select an available language different from the active language in a managed installation.' }
            $availability = Get-LanguageAvailability $languageCodes[$languages.SelectedIndex] $script:configured $script:sourceLanguages $download.Checked
            $downloadRequested = $download.Checked -and -not $availability.AllLocal
        }
        $downloadConfirmed = $false
        if ($downloadRequested) {
            $answer = [Windows.Forms.MessageBox]::Show($form,
                "This downloads EA's original game data (933 MB) from Internet Archive, a third-party service. Availability and integrity checks do not establish permission to use it.`n`nOnly continue if you are entitled to download and use these files. Do you confirm that you are entitled and want to download them?",
                'Confirm game-data download', [Windows.Forms.MessageBoxButtons]::YesNo,
                [Windows.Forms.MessageBoxIcon]::Warning, [Windows.Forms.MessageBoxDefaultButton]::Button2)
            if ($answer -ne [Windows.Forms.DialogResult]::Yes) { return }
            $downloadConfirmed = $true
        }
        $script:currentAction = $action
        $script:currentJob = Join-Path $root ('private\setup-jobs\' + [Guid]::NewGuid().ToString('N'))
        [void][IO.Directory]::CreateDirectory($script:currentJob)
        $jobPath = Join-Path $script:currentJob 'job.json'
        Save-State $jobPath @{
            directory = $script:currentJob
            action = $action
            source = [string]$choices.SelectedItem
            download = $downloadRequested
            downloadConfirmed = $downloadConfirmed
            language = $languageCodes[$languages.SelectedIndex]
        }
        $start = New-Object Diagnostics.ProcessStartInfo
        $start.FileName = Join-Path $PSHOME 'powershell.exe'
        $start.Arguments = '-NoLogo -NoProfile -ExecutionPolicy Bypass -Command "& $env:LG_UI_SCRIPT -Mode Worker -JobFile $env:LG_UI_JOB"'
        $start.EnvironmentVariables['LG_UI_SCRIPT'] = $PSCommandPath
        $start.EnvironmentVariables['LG_UI_JOB'] = $jobPath
        $start.UseShellExecute = $false; $start.CreateNoWindow = $true; $start.WorkingDirectory = $root
        $start.RedirectStandardError = $true
        $script:worker = [Diagnostics.Process]::Start($start)
        $script:workerErrors = $script:worker.StandardError.ReadToEndAsync()
        $statusLabel.Text = if ($action -eq 'language') { 'Changing the installed language...' } else { 'Starting setup...' }
        $progress.Style = 'Marquee'; $cancel.Text = 'Cancel'
        Refresh-Inputs
        $timer.Start()
    }
    $go.Add_Click({
        try {
            if ($script:ready) { Start-Game; $form.Close(); return }
            Start-SetupWorker 'setup'
        } catch { [void][Windows.Forms.MessageBox]::Show($form, $_.Exception.Message, 'Looking Glass - Setup') }
    })
    $applyLanguage.Add_Click({
        try { Start-SetupWorker 'language' }
        catch { [void][Windows.Forms.MessageBox]::Show($form, $_.Exception.Message, 'Looking Glass - Language') }
    })
    function Request-Cancel {
        [IO.File]::WriteAllText((Join-Path $script:currentJob 'cancel'), 'cancel', $utf8)
        $statusLabel.Text = 'Cancelling safely. Waiting for the current file operation to stop...'
        $cancel.Enabled = $false
    }
    $cancel.Add_Click({ if ($script:worker) { Request-Cancel } else { $form.Close() } })
    $form.Add_FormClosing({
        if ($script:worker) { $_.Cancel = $true; $script:closeWhenDone = $true; Request-Cancel }
    })
    $timer.Add_Tick({
        $statePath = Join-Path $script:currentJob 'status.json'
        if (Test-Path -LiteralPath $statePath) {
            try {
                $state = [IO.File]::ReadAllText($statePath) | ConvertFrom-Json
                if ($cancel.Enabled) { $statusLabel.Text = $state.message }
                if ($state.percent -ge 0) { $progress.Style = 'Continuous'; $progress.Value = [Math]::Min(100, $state.percent) }
                else { $progress.Style = 'Marquee' }
            } catch { } # A short status-file race is retried on the next tick.
        }
        if ($script:worker.HasExited) {
            $timer.Stop()
            $code = $script:worker.ExitCode
            $workerError = $script:workerErrors.Result
            $script:worker.Dispose(); $script:worker = $null
            $progress.Style = 'Continuous'; $cancel.Enabled = $true; $cancel.Text = 'Close'
            Refresh-Inputs
            if ($code -eq 0) {
                $script:configured = Read-ConfiguredData
                            if ($script:configured) {
                    # Changing the selection fires a UI refresh, never a worker.
                    $languages.SelectedIndex = [Array]::IndexOf($languageCodes, (Get-ActiveLanguage $script:configured))
                }
                $script:ready = $script:currentAction -eq 'setup' -and (Test-Path -LiteralPath (Join-Path $root 'looking-glass.exe'))
                $script:sourceKey = $null
                Start-SourceScan
                Refresh-Language
                $progress.Value = 100
                $statusLabel.Text = if ($script:currentAction -eq 'language') { 'Language changed. Base game files and settings were kept. Use Launch to play.' } else { 'Ready to play. Next time, just open Launch.' }
                if ($script:currentAction -eq 'setup' -and $desktop.Checked) {
                    try {
                        $linkPath = Join-Path ([Environment]::GetFolderPath('Desktop')) 'Looking Glass.lnk'
                        # Never replace an existing shortcut belonging to another installation.
                        if (Test-Path -LiteralPath $linkPath) { $linkPath = Join-Path ([Environment]::GetFolderPath('Desktop')) ('Looking Glass ' + [Guid]::NewGuid().ToString('N').Substring(0, 6) + '.lnk') }
                        $shell = New-Object -ComObject WScript.Shell
                        $link = $shell.CreateShortcut($linkPath)
                        $link.TargetPath = Join-Path $PSHOME 'powershell.exe'
                        $link.Arguments = '-NoLogo -NoProfile -STA -WindowStyle Hidden -ExecutionPolicy Bypass -File "' + $PSCommandPath + '" -Mode Play'
                        $link.WorkingDirectory = $root; $link.IconLocation = (Join-Path $root 'looking-glass.exe'); $link.Save()
                    } catch { $statusLabel.Text = 'Game files are ready. The desktop shortcut could not be created; use Launch.cmd.' }
                }
                if ($script:currentAction -eq 'setup' -and $playAfter.Checked -and -not $script:closeWhenDone) {
                    try { Start-Game; $form.Close() }
                    catch { $statusLabel.Text = $_.Exception.Message }
                }
            } else {
                if ($workerError) { $statusLabel.Text = 'Setup could not finish. Please try again or choose an existing archive or folder.' }
                $log = Join-Path $script:currentJob 'setup.log'
                if (Test-Path -LiteralPath $log) {
                    $lines = @(Get-Content -LiteralPath $log | Where-Object { $_.Trim() })
                    if ($lines.Count) { $statusLabel.Text = $lines[-1] }
                }
                $progress.Value = 0
            }
            Refresh-Inputs
            if ($script:closeWhenDone) { $form.Close() }
        }
    })
    $selectedBase = Find-Data ([string]$choices.SelectedItem)
    if (-not $script:configured -and $selectedBase) {
        $languages.SelectedIndex = [Array]::IndexOf($languageCodes, (Get-ActiveLanguage $selectedBase $true))
    }
    Start-SourceScan
    Refresh-Language
    [void]$form.ShowDialog()
    $sourceTimer.Stop(); $sourceTimer.Dispose()
    if ($script:sourceScan) { $script:sourceScan.Stop(); $script:sourceScan.Dispose() }
    $timer.Dispose(); $form.Dispose()
} catch {
    [void][Windows.Forms.MessageBox]::Show($_.Exception.Message, 'Looking Glass - Setup')
    exit 1
} finally {
    if ($owned) { $mutex.ReleaseMutex() }
    $mutex.Dispose()
}

if (Test-Path -Path C:\Users\jenkins\.cargo\bin\rustup.exe) {
    exit
}

$client = new-object System.Net.WebClient
$client.DownloadFile('https://win.rustup.rs', "$pwd\rustup-init.exe")
.\rustup-init.exe -y

[cmdletbinding()]
param($tag)


if($null -eq $tag || $tag -eq "") {
    write-error "Specify tag." -ea stop
}

set-location .\Clients\VSCode\bin -ea stop

try {
    
    $osxurl = "https://github.com/OlofBlomqvist/marlowe_lsp/releases/download/$tag/marlowe_lsp_$($tag)_x86_64-apple-darwin.zip"
    $winurl = "https://github.com/OlofBlomqvist/marlowe_lsp/releases/download/$tag/marlowe_lsp_$($tag)_x86_64-pc-windows-gnu.zip"
    $linurl = "https://github.com/OlofBlomqvist/marlowe_lsp/releases/download/$tag/marlowe_lsp_$($tag)_x86_64-unknown-linux-musl.tar.gz"

    remove-item *.gz
    remove-item *.zip
    remove-item *.exe 
    remove-item *.bin

    Invoke-WebRequest -uri $osxurl -OutFile osx.zip -ea stop -verbose
    write-host "OSX - DOWNLOADED FROM $osxurl"
    Invoke-WebRequest -uri $winurl -OutFile win.zip -ea stop -verbose
    write-host "WIN - DOWNLOADED FROM $winurl"
    Invoke-WebRequest -uri $linurl -OutFile lin.tar.gz -ea stop -verbose
    write-host "LIN - DOWNLOADED FROM $linurl"

    $win = Get-ChildItem win.zip
    $osx = Get-ChildItem osx.zip
    $tux = Get-ChildItem lin.tar.gz

    Expand-Archive $win -DestinationPath . -ea stop
    move-item *.exe marlowe_lsp_x86_64-windows.exe -ea stop
    write-host "WIN - EXTRACTED"

    expand-archive $osx -DestinationPath . -ea stop
    move-item marlowe_lsp marlowe_lsp_x86_64-apple-darwin.bin -ea stop
    write-host "OSX - EXTRACTED"

    tar xf "$($tux.fullname)"
    move-item marlowe_lsp marlowe_lsp_x86_64-unknown-linux-musl.bin -ea stop
    write-host "LIN - EXTRACTED"

    remove-item *.gz
    remove-item *.zip
} finally {
    set-location ../../../
}




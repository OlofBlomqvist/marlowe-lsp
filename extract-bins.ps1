
$win = Get-ChildItem *windows-gnu.zip
$osx = Get-ChildItem *apple-darwin.zip
$tux = Get-ChildItem *linux-musl.tar.gz

Remove-Item *.bin
Remove-Item *.exe

Expand-Archive $win -DestinationPath . -ea stop
move-item *.exe marlowe_lsp_x86_64-windows.exe -ea stop

expand-archive $osx -DestinationPath . -ea stop
move-item marlowe_lsp marlowe_lsp_x86_64-apple-darwin.bin -ea stop

tar xf "$($tux.fullname)"
move-item marlowe_lsp marlowe_lsp_x86_64-unknown-linux-musl.bin -ea stop





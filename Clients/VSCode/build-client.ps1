Remove-Item .\dist\* -Recurse
Remove-Item *.vsix
$hash_of_server = python hash.py
$x = Get-Content .\server\src\browserServerMain.ts
if($x -imatch $hash_of_server) {
	write-host -ForegroundColor green "OK HASH MATCH:  $($hash_of_server)"
} else {
	Write-Error -ea stop -Message "did you update the hash? expected $($hash_of_server)"
}

vsce.cmd package
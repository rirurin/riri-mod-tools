# Set Working Directory
Split-Path $MyInvocation.MyCommand.Path | Push-Location
[Environment]::CurrentDirectory = $PWD

Remove-Item "$env:RELOADEDIIMODS/riri.modruntime/*" -Force -Recurse
dotnet publish "./riri.modruntime.csproj" -c Release -o "$env:RELOADEDIIMODS/riri.modruntime" /p:OutputPath="./bin/Release" /p:ReloadedILLink="true"

# Restore Working Directory
Pop-Location
# Helper script to run all examples for quick testing. Could be replaced with something more fancy
# at a later stage.

$baseDir = Split-Path -parent $PSScriptRoot
Get-ChildItem -Directory "$baseDir/guest/rust/examples" |
    Get-ChildItem -Directory |
    % { $_.FullName } |
    Sort-Object |
    % { cargo run -- run "$_" }
# Define the paths
$targetDir = "$env:USERPROFILE\.rhz"
$sourceExe = ".\target\release\rhiza.exe"
$destinationExe = "$targetDir\rhiza.exe"

# Step 1: Create the directory ~/.rhz (if it doesn't exist)
mkdir -Force $targetDir | Out-Null
Write-Host "Created directory (if it didn't exist): $targetDir"

# Step 2: Copy rhiza.exe to ~/.rhz
if (Test-Path $sourceExe) {
    cp -Force $sourceExe $destinationExe
    Write-Host "Copied $sourceExe to $destinationExe"
} else {
    Write-Host "Source file not found: $sourceExe"
    exit 1
}

# Step 3: Add ~/.rhz to the user's PATH
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if (-Not ($userPath -split ';' -contains $targetDir)) {
    $newPath = "$userPath;$targetDir"
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    Write-Host "Added $targetDir to the user's PATH"
} else {
    Write-Host "$targetDir is already in the user's PATH"
}

Write-Host "Installation completed successfully."

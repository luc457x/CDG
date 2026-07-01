# add_git_bash_to_path.ps1 - Automated script to append Git Bash utilities folder to user PATH
# Part of the rtk workspace skill troubleshooting

$gitPath = "C:\Program Files\Git\usr\bin"
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -split ';' -notcontains $gitPath) {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$gitPath", "User")
    Write-Host "Successfully appended Git Bash utilities to user PATH."
} else {
    Write-Host "Git Bash utilities path is already in user PATH."
}

# Update current session PATH immediately
if ($env:Path -split ';' -notcontains $gitPath) {
    $env:Path = "$gitPath;" + $env:Path
    Write-Host "Active session PATH updated."
}


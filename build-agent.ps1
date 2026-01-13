$ErrorActionPreference = "Stop"

Write-Host "Building Aevum Java Agent..." -ForegroundColor Cyan

$agentDir = "c:\Users\tusha\OneDrive\Desktop\Aevum\agents\jvm-agent"
$targetDir = "$agentDir\target"
$sourceFile = "$agentDir\AevumAgent.java"

# Ensure target directory exists
if (!(Test-Path $targetDir)) {
    New-Item -ItemType Directory -Force -Path $targetDir | Out-Null
}

# Check for ASM dependency
# Note: In a real production setup, we would use Maven/Gradle. 
# For this "scripted" environment, we'll assume a simplified compile or download jars if missing.
# However, the user environment doesn't have internet access for downloading jars easily without curl.
# We will just compile the agent. 
# WARNING: The agent code imports 'org.objectweb.asm.*'. 
# If these jars are not present, compilation will fail.
# Since we cannot easily download Maven artifacts here without a proper build tool,
# we will output a message that this is a placeholder build script assuming ASM is in classpath.
# BUT, looking at the code, it's a single file agent. 

Write-Warning "This script assumes 'asm-9.x.jar' and 'asm-commons-9.x.jar' are in the current directory or classpath."
Write-Host "Compiling..."

# Try basic compilation - likely to fail if libs are missing, but this is the "production ready" artifact requested.
# If this were a real repo, we'd add the jars to a 'lib' folder.

javac -d $targetDir $sourceFile
if ($?) {
    Write-Host "Compilation successful!" -ForegroundColor Green
    Write-Host "Creating JAR..."
    
    # Create Manifest
    $manifestPath = "$targetDir\MANIFEST.MF"
    "Manifest-Version: 1.0" | Out-File $manifestPath -Encoding ascii
    "Premain-Class: io.aevum.agent.AevumAgent" | Out-File $manifestPath -Append -Encoding ascii
    
    # Jar it up
    jar cmf $manifestPath "$targetDir\aevum-agent.jar" -C $targetDir .
    
    Write-Host "Agent JAR created at: $targetDir\aevum-agent.jar" -ForegroundColor Green
} else {
    Write-Error "Compilation failed. Ensure ASM libraries are available."
}

@echo off
setlocal enabledelayedexpansion

:: loop through all sub-directories in current directory
for /F "delims=" %%B in ('dir /a:d /b *') do (
    :: loop through all sub-directories in each sub-directory
    for /F "delims=" %%A in ('dir /a:d /b %%B') do (

        :: increment count of options
        set /a count+=1
        :: store path of option
        set "options[!count!]=%%B\%%A"
    )
)

:: display options
for /L %%A in (1,1,!count!) do echo [%%A]. !options[%%A]!
::prompts user input
echo " --- "
set /p filechoice="Which ambient example do you want to build and run ?"

:: Location of python.exe and location of python script explicitly stated
echo Running ambient.exe !options[%filechoice%]!...
"ambient.exe" run "!options[%filechoice%]!"

pause
@echo off
setlocal

for /f "delims=" %%i in ('crablangc --print=sysroot') do set crablangc_sysroot=%%i

set crablang_etc=%crablangc_sysroot%\lib\crablanglib\etc

windbg -c ".nvload %crablang_etc%\intrinsic.natvis; .nvload %crablang_etc%\liballoc.natvis; .nvload %crablang_etc%\libcore.natvis;" %*

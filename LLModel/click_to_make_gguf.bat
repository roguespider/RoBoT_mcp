@echo off
title Qwen 3.6 MoE Local End-to-End Automated Coder Abliteration and Quantization Pipeline
cls

echo ============================================================
echo   RUNNING FULL NATIVE QWEN 3.6 CODER MOE PIPELINE (Q4 GENERATOR)
echo ============================================================

:: Forcefully clear bad global environment overrides causing pip index errors
set "PIP_INDEX_URL="
set "PIP_EXTRA_INDEX_URL="
set "UV_INDEX_URL="

:: Lock execution focus strictly to your E: Drive target directory
cd /d "E:\Tor\llm"

:: CONFIGURATION MATRIX
set "CACHE_DIR=E:\Tor\llm\models--Qwen--Qwen3.6-35B-A3B\snapshots\995ad96eacd98c81ed38be0c5b274b04031597b0"
set "TEMP_HF_OUT=E:\Tor\llm\temp_abliterated_hf"
set "FINAL_Q4_GGUF=E:\Tor\llm\qwen3.6-35b-a3b-code-abliterated-q4_k_m.gguf"
set "LMS_PYTHON=venv_312\Scripts\python.exe"

:: ============================================================
:: VALIDATION ENGINE & AUTOMATED DEPENDENCY FETCH
:: ============================================================
if not exist "%CACHE_DIR%" (
    echo.
    echo ------------------------------------------------------------
    echo   [NOTICE] Model not found locally at %CACHE_DIR%
    echo   Downloading from HuggingFace automatically...
    echo ------------------------------------------------------------
    :: Download model using huggingface_hub Python API
    py -m pip install huggingface_hub --quiet
    if errorlevel 1 (
        echo [ERROR] Failed to install huggingface_hub.
        pause
        exit /b
    )
    py -c "from huggingface_hub import snapshot_download; snapshot_download('Qwen/Qwen3.6-35B-A3B', local_dir=r'%CACHE_DIR%', force_download=True)"
    if errorlevel 1 (
        echo [ERROR] Model download failed. Check your internet connection.
        pause
        exit /b
    )
    echo [SUCCESS] Model downloaded to %CACHE_DIR%
) else (
    echo [OK] Model found locally at %CACHE_DIR%
)

:: AUTOMATED TOOL SETUP: Pull llama-quantize if missing
if not exist "llama-quantize.exe" (
    echo.
    echo ------------------------------------------------------------
    echo   [NOTICE] llama-quantize.exe is missing from E:\Tor\llm
    echo   Automatically downloading the pre-built windows binary...
    echo ------------------------------------------------------------
    powershell -Command "$ProgressPreference = 'SilentlyContinue'; [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; $releases = Invoke-RestMethod -Uri 'https://api.github.com/repos/ggml-org/llama.cpp/releases/latest'; $cuda13 = $releases.assets | Where-Object {$_.name -like 'llama-*-bin-win-cuda-13.3*x64.zip'} | Select-Object -First 1; $cuda12 = $releases.assets | Where-Object {$_.name -like 'llama-*-bin-win-cuda-12.4*x64.zip'} | Select-Object -First 1; $bin = if ($cuda13) { $cuda13 } else { $cuda12 }; if ($bin) { Invoke-WebRequest -Uri $bin.browser_download_url -OutFile 'llama_bin.zip'; Expand-Archive -Path 'llama_bin.zip' -DestinationPath 'llama_temp' -Force; Get-ChildItem -Recurse -Path 'llama_temp' -Filter 'llama-quantize.exe' | Move-Item -Destination . -Force; Remove-Item -Path 'llama_temp' -Recurse -Force; Remove-Item -Path 'llama_bin.zip' -Force } else { exit 1 }"

    :: Secondary verification check
    if not exist "llama-quantize.exe" (
        echo [ERROR] Automated engine download failed.
        echo Please drop a copy of llama-quantize.exe manually inside E:\Tor\llm to continue.
        pause
        exit /b
    )
)

:: Ensure temporary HF output path exists
if not exist "%TEMP_HF_OUT%" (mkdir "%TEMP_HF_OUT%" && echo [SUCCESS] Temporary output directory created. || echo [FAILED] Failed to create temporary output directory.)

:: ============================================================
:: STAGE 0: AUTOMATED ENVIRONMENT CONSTRUCTOR
:: ============================================================
if not exist "venv_312" (
    echo.
    echo ============================================================
    echo   INITIALIZING FRESH VIRTUAL WORKSPACE CONTAINER
    echo ============================================================
    echo -> Creating venv_312 runtime architecture...
    py -3.12 -m venv venv_312 2>nul || python -m venv venv_312
    if errorlevel 1 (echo [FAILED] Failed to create virtual environment. Aborting.& goto FAILED) else (echo [SUCCESS] Virtual environment created.)
)

echo -> Upgrading core packaging systems...
"%LMS_PYTHON%" -m pip install --upgrade pip
if errorlevel 1 (echo [FAILED] Failed to upgrade pip. Aborting.& goto FAILED) else (echo [SUCCESS] pip upgraded successfully.)

echo -> Installing pipeline compute dependencies...
"%LMS_PYTHON%" -m pip install torch==2.11.0+cu128 torchvision --index-url https://download.pytorch.org/whl/cu128 --no-cache-dir
if errorlevel 1 (echo [FAILED] Failed to install PyTorch. Aborting.& goto FAILED) else (echo [SUCCESS] PyTorch installed successfully.)

"%LMS_PYTHON%" -m pip install "transformers>=5.3.0" accelerate safetensors gguf sentencepiece
if errorlevel 1 (echo [FAILED] Failed to install transformers and dependencies. Aborting.& goto FAILED) else (echo [SUCCESS] Transformers and dependencies installed successfully.)

:: ============================================================
:: STAGE 1: NATIVE MOE MATRIX ABLITERATION CHANNELS
:: ============================================================
echo.
echo ============================================================
echo   RUNNING MATRIX ABLITERATION CHANNELS
echo ============================================================
echo -> Assembling optimized local Python tensor alignment script...
powershell -ExecutionPolicy Bypass -File "gen_pipeline.ps1" -CacheDir "%CACHE_DIR%" -OutHfPath "%TEMP_HF_OUT%"
if errorlevel 1 (echo [FAILED] Failed to write pipeline script. Aborting.& goto FAILED) else (echo [SUCCESS] Pipeline script generated.)

echo.
echo [Invoking Runtime Matrix Container Engine...]
"%LMS_PYTHON%" execute_local_pipeline.py
if errorlevel 1 (echo [FAILED] Abliteration pipeline script terminated with an error. Aborting.& goto FAILED) else (echo [SUCCESS] Abliteration pipeline completed successfully.)
if exist "execute_local_pipeline.py" del /f /q "execute_local_pipeline.py"

:: ============================================================
:: STAGE 2: GGUF MATRIX QUANTIZATION ENGINE
:: ============================================================
echo.
echo ============================================================
echo   CONVERTING UNQUANTIZED MODEL BACKBONE TO BASE GGUF TEMPLATE
echo ============================================================

:: Download full llama.cpp source if conversion tools missing
if exist "llama.cpp" (
    echo [OK] llama.cpp already exists.
) else (
    echo -> Downloading full llama.cpp source (requires conversion modules)...
    powershell -Command "Invoke-WebRequest -Uri 'https://github.com/ggml-org/llama.cpp/archive/refs/heads/master.zip' -OutFile 'llama_cpp.zip'"
    if errorlevel 1 (echo [FAILED] Failed to download llama.cpp source. Aborting.& goto FAILED) else (echo [SUCCESS] llama.cpp source downloaded.)
    echo -> Extracting llama.cpp source...
    powershell -Command "Expand-Archive -Path 'llama_cpp.zip' -DestinationPath '.' -Force"
    if errorlevel 1 (echo [FAILED] Failed to extract llama.cpp source. Aborting.& goto FAILED) else (echo [SUCCESS] llama.cpp source extracted.)
    if exist 'llama_cpp.zip' del /f /q 'llama_cpp.zip'
    :: Rename extracted folder to llama.cpp
    if exist 'llama.cpp-master' (
        move 'llama.cpp-master' 'llama.cpp' >nul 2>&1
    )
    if not exist "llama.cpp" (
        :: Find any extracted directory containing the conversion script
        for /d %%d in (llama.cpp-*) do if exist "%%d\convert_hf_to_gguf.py" (
            move "%%d" "llama.cpp"
            goto llama_cpp_ready
        )
        :llama_cpp_not_found
        echo [FAILED] Could not find convert_hf_to_gguf.py in extracted llama.cpp.
        echo Please ensure the downloaded archive contains the llama.cpp source.
        goto FAILED
    )
    :llama_cpp_ready
    echo [SUCCESS] llama.cpp source ready for conversion.
)

echo -> Converting HF weights to f16 GGUF...
cd /d "E:\Tor\llm\llama.cpp"
"%LMS_PYTHON%" convert_hf_to_gguf.py "%TEMP_HF_OUT%" --outfile "E:\Tor\llm\%FINAL_Q4_GGUF%.f16"
if errorlevel 1 (echo [FAILED] HF to GGUF conversion failed. Aborting.& goto FAILED) else (echo [SUCCESS] HF to GGUF conversion completed.)
cd /d "E:\Tor\llm"

:: Clean up llama.cpp source after conversion
if exist "llama.cpp" (rmdir /s /q "llama.cpp" && echo [SUCCESS] llama.cpp source cleaned up. || echo [FAILED] Failed to clean up llama.cpp source.)

:: ============================================================
:: STAGE 3: Q4_K_M QUANTIZATION
:: ============================================================
echo.
echo ============================================================
echo   QUANTIZING GGUF TO Q4_K_M
:: ============================================================
if not exist "llama-quantize.exe" (
    echo [ERROR] llama-quantize.exe not found. Cannot quantize.
    goto FAILED
)
echo -> Quantizing f16 GGUF to Q4_K_M...
"llama-quantize.exe" "E:\Tor\llm\%FINAL_Q4_GGUF%.f16" "E:\Tor\llm\%FINAL_Q4_GGUF%" Q4_K_M
if errorlevel 1 (echo [FAILED] Quantization failed. Aborting.& goto FAILED) else (echo [SUCCESS] Q4_K_M quantization completed.)
:: Remove the f16 intermediate file
if exist "E:\Tor\llm\%FINAL_Q4_GGUF%.f16" del /f /q "E:\Tor\llm\%FINAL_Q4_GGUF%.f16"

echo.
echo.
echo ============================================================
echo   [SUCCESS] PIPELINE COMPLETE! Q4_K_M GGUF READY!
echo ============================================================
echo   Output: %FINAL_Q4_GGUF%
echo ============================================================
echo.
:: Cleanup prompt
set /p "CHOICE=Delete temporary abliterated HF weights? [Y/N]: "
if /i "%CHOICE%"=="Y" (
    if exist "%TEMP_HF_OUT%" rmdir /s /q "%TEMP_HF_OUT%"
    echo -> Temp folder deleted.
) else (
    echo -> Keeping temp folder at: %TEMP_HF_OUT%
)
pause
exit /b

:FAILED
echo.
echo ============================================================
echo   [FATAL ERROR] PIPELINE TERMINATED WITH A FAILURE CODE
echo ============================================================
if exist "execute_local_pipeline.py" del /f /q "execute_local_pipeline.py"
pause
exit /b

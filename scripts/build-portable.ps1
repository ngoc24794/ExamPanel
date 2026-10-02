# ==============================================================================
# Script đóng gói phiên bản ExamPanel Portable cho Windows
# Sử dụng:
#   .\scripts\build-portable.ps1              (Build mới rồi đóng gói)
#   .\scripts\build-portable.ps1 -SkipBuild   (Chỉ đóng gói từ bản release đã build sẵn)
# ==============================================================================

param (
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "   BẮT ĐẦU ĐÓNG GÓI EXAMPANEL PHIÊN BẢN PORTABLE (WINDOWS)" -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Biên dịch ứng dụng nếu không bật -SkipBuild
if (-not $SkipBuild) {
    Write-Host "`n[1/4] Đang biên dịch frontend và ứng dụng Tauri (Release)..." -ForegroundColor Yellow
    pnpm tauri build
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Lỗi: Quá trình pnpm tauri build thất bại!"
        exit 1
    }
} else {
    Write-Host "`n[1/4] Bỏ qua bước biên dịch (-SkipBuild)..." -ForegroundColor Gray
}

# 2. Xác định file thực thi nguồn
$srcExe = $null
if (Test-Path "target/release/exam-panel-app.exe") {
    $srcExe = "target/release/exam-panel-app.exe"
} elseif (Test-Path "target/release/ExamPanel.exe") {
    $srcExe = "target/release/ExamPanel.exe"
} else {
    Write-Error "Lỗi: Không tìm thấy file thực thi trong target/release/ (exam-panel-app.exe hoặc ExamPanel.exe)!"
    exit 1
}

Write-Host "`n[2/4] Chuẩn bị thư mục target/portable/..." -ForegroundColor Yellow
$portableDir = "target/portable"

# Xóa nội dung cũ trong thư mục portable nếu có để đảm bảo file luôn mới nhất
if (Test-Path $portableDir) {
    Remove-Item -Path "$portableDir/*" -Recurse -Force -ErrorAction SilentlyContinue
} else {
    New-Item -ItemType Directory -Force -Path $portableDir | Out-Null
}

# 3. Sao chép các tệp thành phần vào target/portable/
Write-Host "`n[3/4] Sao chép các tệp thành phần vào $portableDir/..." -ForegroundColor Yellow

# Copy file exe sang tên chính thức ExamPanel.exe
Copy-Item $srcExe -Destination "$portableDir/ExamPanel.exe" -Force
Write-Host "  -> Đã sao chép: ExamPanel.exe ($((Get-Item "$portableDir/ExamPanel.exe").Length / 1MB | ForEach-Object { '{0:N2} MB' -f $_ }))" -ForegroundColor Green

# Tạo file marker ExamPanel.portable
New-Item -ItemType File -Force -Path "$portableDir/ExamPanel.portable" | Out-Null
Write-Host "  -> Đã tạo marker: ExamPanel.portable (kích hoạt chế độ di động)" -ForegroundColor Green

# Copy hướng dẫn sử dụng DOC-TOI.txt
if (Test-Path "DOC-TOI.txt") {
    Copy-Item "DOC-TOI.txt" -Destination "$portableDir/DOC-TOI.txt" -Force
    Write-Host "  -> Đã sao chép: DOC-TOI.txt" -ForegroundColor Green
}

# Copy giấy phép bên thứ ba
if (Test-Path "THIRD_PARTY_NOTICES") {
    Copy-Item "THIRD_PARTY_NOTICES" -Destination "$portableDir/THIRD_PARTY_NOTICES.txt" -Force
    Write-Host "  -> Đã sao chép: THIRD_PARTY_NOTICES.txt" -ForegroundColor Green
}

# 4. Nén thành tệp ZIP hoàn chỉnh
Write-Host "`n[4/4] Nén thư mục portable thành tệp ZIP..." -ForegroundColor Yellow
$zipPath = "target/ExamPanel-0.1.0-windows-x64-portable.zip"
if (Test-Path $zipPath) {
    Remove-Item $zipPath -Force
}
Compress-Archive -Path "$portableDir/*" -DestinationPath $zipPath -Force
Write-Host "  -> Đã tạo ZIP: $zipPath ($((Get-Item $zipPath).Length / 1MB | ForEach-Object { '{0:N2} MB' -f $_ }))" -ForegroundColor Green

Write-Host "`n==========================================================" -ForegroundColor Cyan
Write-Host " HOÀN TẤT! THƯ MỤC VÀ TỆP PORTABLE ĐÃ SẴN SÀNG:" -ForegroundColor Green
Write-Host " - Thư mục chạy trực tiếp: $(Resolve-Path $portableDir)\ExamPanel.exe" -ForegroundColor White
Write-Host " - Tệp nén mang đi:        $(Resolve-Path $zipPath)" -ForegroundColor White
Write-Host "==========================================================" -ForegroundColor Cyan

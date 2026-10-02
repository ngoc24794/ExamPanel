#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { execSync } from 'node:child_process';

const skipBuild = process.argv.includes('--skip-build') || process.argv.includes('-SkipBuild');

console.log('==========================================================');
console.log('   BẮT ĐẦU ĐÓNG GÓI EXAMPANEL PHIÊN BẢN PORTABLE (macOS)');
console.log('==========================================================');

const pkgJson = JSON.parse(fs.readFileSync('package.json', 'utf8'));
const version = pkgJson.version || '0.1.0';

if (!skipBuild) {
  console.log('\n[1/4] Đang biên dịch frontend và ứng dụng Tauri (Release cho macOS)...');
  execSync('pnpm tauri build', { stdio: 'inherit' });
} else {
  console.log('\n[1/4] Bỏ qua bước biên dịch (--skip-build)...');
}

const candidates = [
  'target/release/bundle/macos/ExamPanel.app',
  'src-tauri/target/release/bundle/macos/ExamPanel.app',
  'target/universal-apple-darwin/release/bundle/macos/ExamPanel.app',
  'target/aarch64-apple-darwin/release/bundle/macos/ExamPanel.app',
  'target/x86_64-apple-darwin/release/bundle/macos/ExamPanel.app',
  'src-tauri/target/universal-apple-darwin/release/bundle/macos/ExamPanel.app',
  'src-tauri/target/aarch64-apple-darwin/release/bundle/macos/ExamPanel.app',
  'src-tauri/target/x86_64-apple-darwin/release/bundle/macos/ExamPanel.app',
];

const appSrc = candidates.find((p) => fs.existsSync(p));
if (!appSrc) {
  console.error('Lỗi: Không tìm thấy ExamPanel.app trong target/.../bundle/macos/!');
  process.exit(1);
}

console.log(`  -> Đã tìm thấy ứng dụng nguồn: ${appSrc}`);

console.log('\n[2/4] Chuẩn bị thư mục target/portable-macos/...');
const portableDir = path.join('target', 'portable-macos', 'ExamPanel-portable');
if (fs.existsSync('target/portable-macos')) {
  fs.rmSync('target/portable-macos', { recursive: true, force: true });
}
fs.mkdirSync(portableDir, { recursive: true });

console.log(`\n[3/4] Sao chép các tệp thành phần vào ${portableDir}/...`);
const destApp = path.join(portableDir, 'ExamPanel.app');

try {
  execSync(`ditto "${appSrc}" "${destApp}"`);
} catch {
  execSync(`cp -R "${appSrc}" "${destApp}"`);
}
console.log('  -> Đã sao chép: ExamPanel.app');

// Marker in root and inside bundle
fs.writeFileSync(path.join(portableDir, 'ExamPanel.portable'), '');
const innerMacOS = path.join(destApp, 'Contents', 'MacOS');
if (fs.existsSync(innerMacOS)) {
  fs.writeFileSync(path.join(innerMacOS, 'ExamPanel.portable'), '');
}
console.log('  -> Đã tạo marker: ExamPanel.portable');

if (fs.existsSync('DOC-TOI.txt')) {
  fs.copyFileSync('DOC-TOI.txt', path.join(portableDir, 'DOC-TOI.txt'));
  console.log('  -> Đã sao chép: DOC-TOI.txt');
}

if (fs.existsSync('THIRD_PARTY_NOTICES')) {
  fs.copyFileSync('THIRD_PARTY_NOTICES', path.join(portableDir, 'THIRD_PARTY_NOTICES.txt'));
  console.log('  -> Đã sao chép: THIRD_PARTY_NOTICES.txt');
}

console.log('\n[4/4] Nén thư mục portable thành tệp lưu hành...');
const tarFile = `target/ExamPanel-${version}-macos-portable.tar.gz`;
const zipFile = `target/ExamPanel-${version}-macos-portable.zip`;

try {
  execSync(`tar -czf "../../${tarFile}" ExamPanel-portable`, {
    cwd: path.join('target', 'portable-macos'),
    stdio: 'inherit',
  });
  console.log(`  -> Đã tạo tệp nén TAR.GZ: ${tarFile}`);
} catch (e) {
  console.warn('  -> Cảnh báo: Lệnh tar thất bại:', e.message);
}

try {
  execSync(`ditto -c -k --sequesterRsrc --keepParent "${portableDir}" "${zipFile}"`);
  console.log(`  -> Đã tạo tệp nén ZIP: ${zipFile}`);
} catch {
  try {
    execSync(`zip -q -r -y "../../${zipFile}" ExamPanel-portable`, {
      cwd: path.join('target', 'portable-macos'),
    });
    console.log(`  -> Đã tạo tệp nén ZIP: ${zipFile}`);
  } catch (err) {
    console.warn('  -> Cảnh báo: Lệnh zip thất bại:', err.message);
  }
}

console.log('\n==========================================================');
console.log(' HOÀN TẤT! BẢN PORTABLE CHO MACOS ĐÃ SẴN SÀNG');
console.log(` - Thư mục chạy: ${destApp}`);
console.log(` - Tệp gói:     ${tarFile}`);
console.log('==========================================================');
